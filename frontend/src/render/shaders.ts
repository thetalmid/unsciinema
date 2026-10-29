/**
 * GLSL ES 3.00 shader sources for the WebGL2 preview.
 *
 * Design notes (see INTEGRATION.md §Renderer):
 * - The glyph pass is a *faithful GPU port* of Rust `raster_cells`: identical
 *   operation order, identical scales (RGB in 0..255, alpha in 0..1), and
 *   identical rounding (`round().clamp()`), which is only possible because
 *   atlas/glyph/fg/bg are read through INTEGER textures (R8UI / R16UI /
 *   R32UI) so every byte is bit-identical to the CPU path before math.
 * - ASCII/original blend uses explicit endpoint branches so
 *   `asciiStrength = 1` is bitwise-exact ASCII and `0` is bitwise-exact
 *   source; intermediate values may differ from the CPU by <= 1/255
 *   (documented; endpoints are what the parity test asserts).
 * - All CRT effects are gated: with every control at 0 the post pass is an
 *   exact passthrough (the curvature branch skips its fp roundtrip).
 */

/** Fullscreen triangle-free quad vertex shader. */
export const QUAD_VS = `#version 300 es
precision highp float;
in vec2 a_pos;
out vec2 v_uv;
void main() {
  v_uv = a_pos * 0.5 + 0.5;
  gl_Position = vec4(a_pos, 0.0, 1.0);
}
`;

/**
 * Glyph pass: integer-correct port of `raster_cells` + presentation blend.
 *
 * FBO-local coordinates are bottom-left (GL). All contract math happens in
 * top-left space (`yTop`), matching CPU row-major/top-left output; the final
 * readback flips rows.
 */
export const GLYPH_FS = `#version 300 es
precision highp float;
precision highp int;
precision highp usampler2D;
precision highp usampler2D;

uniform ivec2 u_outSize;      // outputWidth, outputHeight (native)
uniform int u_glyphW;         // glyph pixel width
uniform int u_glyphH;         // glyph pixel height
uniform usampler2D u_glyphTex;  // cols x rows R32UI: atlas index per cell
uniform usampler2D u_atlasTex;  // (16*glyphW) x glyphH R8UI: coverage
uniform usampler2D u_fgTex;     // cols x rows R32UI: packed fg 0xRRGGBBAA
uniform usampler2D u_bgTex;     // cols x rows R32UI: packed bg 0xRRGGBBAA
uniform sampler2D u_srcTex;      // outputW x outputH RGBA8 source (top-left)
uniform float u_asciiStrength;   // 0 = original, 1 = full ASCII

out vec4 fragColor;

// Rust clamp_round(): v.round().clamp(0, 255)  — round half away from zero.
float clampRound(float v) {
  return floor(clamp(v, 0.0, 255.0) + 0.5);
}

void main() {
  ivec2 px = ivec2(gl_FragCoord.xy);
  int yTop = u_outSize.y - 1 - px.y;          // contract space: top-left
  ivec2 cell = ivec2(px.x / u_glyphW, yTop / u_glyphH);

  uint glyph = texelFetch(u_glyphTex, cell, 0).r;
  ivec2 aco = ivec2(int(glyph) * u_glyphW + (px.x - (px.x / u_glyphW) * u_glyphW),
                    yTop - (yTop / u_glyphH) * u_glyphH);
  float ink = float(texelFetch(u_atlasTex, aco, 0).r) / 255.0;

  uint fp = texelFetch(u_fgTex, cell, 0).r;
  uint bp = texelFetch(u_bgTex, cell, 0).r;
  // Identical scales to the CPU reference:
  float fr = float((fp >> 24) & 255u);
  float fg_ = float((fp >> 16) & 255u);
  float fb = float((fp >> 8) & 255u);
  float fa = float(fp & 255u) / 255.0;
  float br = float((bp >> 24) & 255u);
  float bg_ = float((bp >> 16) & 255u);
  float bb = float((bp >> 8) & 255u);
  float ba = float(bp & 255u) / 255.0;

  // Exact port of raster_cells' straight-alpha compositing:
  float la = fa * ink;
  float a = la + ba * (1.0 - la);
  float r, g, b;
  if (a <= 0.0) {
    r = 0.0; g = 0.0; b = 0.0;
  } else {
    r = (fr * la + br * ba * (1.0 - la)) / a;
    g = (fg_ * la + bg_ * ba * (1.0 - la)) / a;
    b = (fb * la + bb * ba * (1.0 - la)) / a;
  }
  vec3 ascii = vec3(clampRound(r), clampRound(g), clampRound(b)) / 255.0;
  vec4 asciiCol = vec4(ascii, clampRound(a * 255.0) / 255.0);

  // Presentation blend with exact endpoints (see header).
  vec4 srcCol = texelFetch(u_srcTex, ivec2(px.x, yTop), 0);
  vec4 color;
  if (u_asciiStrength >= 1.0) {
    color = asciiCol;
  } else if (u_asciiStrength <= 0.0) {
    color = srcCol;
  } else {
    vec3 m = mix(srcCol.rgb, asciiCol.rgb, u_asciiStrength);
    // Same rounding family as CPU raster_blended (half up, clamped).
    color = vec4(floor(clamp(m, 0.0, 1.0) * 255.0 + 0.5) / 255.0,
                 mix(srcCol.a, asciiCol.a, u_asciiStrength));
  }
  fragColor = color;
}
`;

/** Bloom bright-pass + horizontal blur (runs only when bloom > 0). */
export const BLOOM_H_FS = `#version 300 es
precision highp float;
uniform sampler2D u_scene;
uniform vec2 u_texel;       // 1/sceneSize
uniform float u_threshold;
in vec2 v_uv;
out vec4 fragColor;
void main() {
  float w[3] = float[3](0.383, 0.236, 0.011); // gaussian sigma ~1.6
  vec3 acc = texture(u_scene, v_uv).rgb * w[0];
  for (int i = 1; i < 3; i++) {
    vec2 o = vec2(float(i) * u_texel.x, 0.0);
    acc += texture(u_scene, v_uv + o).rgb * w[i];
    acc += texture(u_scene, v_uv - o).rgb * w[i];
  }
  vec3 bright = max(acc - u_threshold, 0.0) * 1.6;
  fragColor = vec4(bright, 1.0);
}
`;

/** Vertical blur (bloom second pass). */
export const BLOOM_V_FS = `#version 300 es
precision highp float;
uniform sampler2D u_input;
uniform vec2 u_texel;       // 1/inputSize
in vec2 v_uv;
out vec4 fragColor;
void main() {
  float w[3] = float[3](0.383, 0.236, 0.011);
  vec3 acc = texture(u_input, v_uv).rgb * w[0];
  for (int i = 1; i < 3; i++) {
    vec2 o = vec2(0.0, float(i) * u_texel.y);
    acc += texture(u_input, v_uv + o).rgb * w[i];
    acc += texture(u_input, v_uv - o).rgb * w[i];
  }
  fragColor = vec4(acc, 1.0);
}
`;

/**
 * Post pass: scene (+bloom) -> CRT effects -> offscreen post FBO at native
 * resolution (the export source). Every effect is gated so 0 is exactly
 * neutral, including the curvature fp roundtrip.
 */
export const POST_FS = `#version 300 es
precision highp float;
uniform sampler2D u_scene;
uniform sampler2D u_bloom;
uniform float u_scanlines;   // 0..1
uniform float u_curvature;   // 0..1
uniform float u_bloomAmt;    // 0..1
uniform float u_vignette;    // 0..1
in vec2 v_uv;
out vec4 fragColor;
void main() {
  vec2 c = v_uv - 0.5;
  vec2 uv = v_uv;
  if (u_curvature > 0.0) {
    uv = 0.5 + c + u_curvature * c * dot(c, c) * 0.6;
    if (any(lessThan(uv, vec2(0.0))) || any(greaterThan(uv, vec2(1.0)))) {
      fragColor = vec4(0.0);
      return;
    }
  }
  vec4 color = texture(u_scene, uv);
  if (u_bloomAmt > 0.0) {
    color.rgb += texture(u_bloom, uv).rgb * u_bloomAmt;
  }
  if (u_scanlines > 0.0) {
    // Darken alternating native-resolution rows; neutral at 0.
    float row = floor(gl_FragCoord.y);
    color.rgb *= 1.0 - u_scanlines * 0.45 * mod(row, 2.0);
  }
  if (u_vignette > 0.0) {
    float d = length(c);
    color.rgb *= 1.0 - u_vignette * 0.9 * smoothstep(0.3, 0.9, d);
  }
  color.rgb = clamp(color.rgb, 0.0, 1.0);
  fragColor = color;
}
`;

/** Final blit: native post FBO -> canvas (NEAREST keeps pixels crisp). */
export const BLIT_FS = `#version 300 es
precision highp float;
uniform sampler2D u_post;
in vec2 v_uv;
out vec4 fragColor;
void main() {
  fragColor = texture(u_post, v_uv);
}
`;
