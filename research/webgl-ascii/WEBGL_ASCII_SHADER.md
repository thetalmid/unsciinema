# WebGL / WGSL ASCII Art Rendering

## Overview
Real-time ASCII rendering using WebGL compute shaders (or fragment shaders) with a font atlas texture. Based on the `bdip-core` WGSL implementation and Efecto's approach.

## Shader Architecture (from bdip-core)

### Two-Pass Approach
1. **Pass 1**: Convert source image to grayscale (BT.709 luma) — stored in scratch texture
2. **Pass 2**: Character-cell rendering with atlas lookup

### Bindings (WGSL)
```wgsl
@group(0) @binding(0) var source_texture: texture_2d<f32>;      // Original color
@group(0) @binding(1) var gray_texture: texture_2d<f32>;        // BT.709 luma
@group(0) @binding(2) var output_texture: texture_storage_2d<rgba16float, write>;
@group(1) @binding(0) var<uniform> params: AsciiArtParams;
@group(2) @binding(0) var char_atlas: texture_2d<f32>;          // 128×8 atlas (16 chars × 8px)
@group(2) @binding(1) var atlas_sampler: sampler;
```

### Uniform Parameters
```wgsl
struct AsciiArtParams {
    cell_size: f32,      // Pixel size of each character cell
    strength: f32,       // 0.0 = original image, 1.0 = full ASCII
    _padding: vec2<f32>,
}
```

### Font Atlas
- **128×8 texture** (16 characters × 8px wide, 1 row × 8px high)
- Each character = 8×8 bitmap
- UV coordinates: `u = (char_idx * 8 + sub_x) / 128`, `v = sub_y / 8`
- Nearest-neighbor sampling for crisp pixel art
- Ink = white (1.0), Background = black (0.0)

### Character Mapping (Density-based)
```wgsl
// 16 characters ordered by visual density (sparse → dense)
const CHAR_RAMP: [char; 16] = [
    ' ', '.', ':', '-', '=', '+', '*', '#', '%', '@', 
    '8', '&', 'W', 'M', '#', '@'  // Example ramp
];

// Luma [0,1] → character index [0,15]
// Invert: bright areas → dense chars, dark areas → sparse chars
let density = 1.0 - clamp(luma, 0.0, 1.0);
let char_idx = floor(density * 15.0 + 0.5);
```

### Rendering Algorithm (Compute Shader)
```wgsl
@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let dims = textureDimensions(source_texture);
    if global_id.x >= dims.x || global_id.y >= dims.y { return; }

    let coord = vec2<i32>(global_id.xy);
    let fcoord = vec2<f32>(global_id.xy);
    let cs = max(params.cell_size, 1.0);

    // 1. Cell origin (top-left)
    let cell_xy = floor(fcoord / cs) * cs;
    // 2. Cell center (for sampling luma & color)
    let cell_ctr = cell_xy + vec2<f32>(cs * 0.5, cs * 0.5);
    let ctr_coord = vec2<i32>(clamp(cell_ctr, vec2<f32>(0.0), vec2<f32>(dims) - 1.0));

    // 3. Sample luma at cell center
    let luma = textureLoad(gray_texture, ctr_coord, 0).r;

    // 4. Map to character index
    let density = 1.0 - clamp(luma, 0.0, 1.0);
    let char_idx = floor(density * 15.0 + 0.5);

    // 5. Sub-pixel within cell [0,7]
    let sub = fcoord - cell_xy;
    let sub_norm = sub / cs;
    let sub_px = floor(sub_norm * 8.0);  // CHAR_PIX = 8.0

    // 6. Sample atlas
    let atlas_u = (char_idx * 8.0 + sub_px.x + 0.5) / 128.0;
    let atlas_v = (sub_px.y + 0.5) / 8.0;
    let ink_mask = textureSampleLevel(char_atlas, atlas_sampler, vec2<f32>(atlas_u, atlas_v), 0.0).r;

    // 7. Compose
    let src_pixel = textureLoad(source_texture, coord, 0);
    let src_ctr = textureLoad(source_texture, ctr_coord, 0);
    let ink_col = src_ctr.rgb * 1.1;      // Tinted cell color
    let bg_col = src_ctr.rgb * 0.05;      // Dark background
    let ascii_col = select(bg_col, ink_col, ink_mask > 0.5);
    let out_rgb = mix(src_pixel.rgb, ascii_col, params.strength);

    textureStore(output_texture, coord, vec4<f32>(out_rgb, src_pixel.a));
}
```

## Alternative: Fragment Shader Approach (for simpler setup)
```glsl
// Vertex shader: full-screen quad
// Fragment shader: per-pixel ASCII lookup
uniform sampler2D uSource;
uniform sampler2D uGray;
uniform sampler2D uAtlas;
uniform float uCellSize;
uniform float uStrength;

void main() {
    vec2 uv = vUv;
    vec2 texSize = textureSize(uSource, 0);
    vec2 coord = uv * texSize;
    float cs = max(uCellSize, 1.0);

    vec2 cell_xy = floor(coord / cs) * cs;
    vec2 cell_ctr = cell_xy + cs * 0.5;
    vec2 ctr_uv = clamp(cell_ctr, vec2(0.0), texSize - 1.0) / texSize;

    float luma = texture(uGray, ctr_uv).r;
    float density = 1.0 - clamp(luma, 0.0, 1.0);
    float char_idx = floor(density * 15.0 + 0.5);

    vec2 sub = coord - cell_xy;
    vec2 sub_norm = sub / cs;
    vec2 sub_px = floor(sub_norm * 8.0);

    float atlas_u = (char_idx * 8.0 + sub_px.x + 0.5) / 128.0;
    float atlas_v = (sub_px.y + 0.5) / 8.0;
    float ink = texture(uAtlas, vec2(atlas_u, atlas_v)).r;

    vec4 src = texture(uSource, uv);
    vec4 src_ctr = texture(uSource, ctr_uv);
    vec3 ink_col = src_ctr.rgb * 1.1;
    vec3 bg_col = src_ctr.rgb * 0.05;
    vec3 ascii_col = mix(bg_col, ink_col, step(0.5, ink));
    vec3 out_rgb = mix(src.rgb, ascii_col, uStrength);

    gl_FragColor = vec4(out_rgb, src.a);
}
```

## Generating the Font Atlas (Rust side)
```rust
fn generate_atlas(font_hex: &str) -> Vec<u8> {
    // Parse unscii-8.hex (8×8 glyphs)
    // Select 16 characters by density
    // Render each 8×8 glyph to a 128×8 texture
    // White = ink (1.0), Black = background (0.0)
    // Order: space, ., :, -, =, +, *, #, %, @, 8, &, W, M, #, @, █
}
```

## Density-Ordered Character Selection (from unscii-8)
| Index | Char | Hex (first 8 bytes) | Visual Density |
|-------|------|---------------------|----------------|
| 0 | ` ` | `0000000000000000` | 0% (empty) |
| 1 | `.` | `0000000000001818` | ~3% |
| 2 | `:` | `0018180000181800` | ~6% |
| 3 | `,` | `0018180000181830` | ~9% |
| 4 | `'` | `0018181818000000` | ~12% |
| 5 | `-` | `0000007E00000000` | ~15% |
| 6 | `+` | `0018187E18180000` | ~25% |
| 7 | `:` | `0018187E18180000` | ~30% |
| 8 | `=` | `00007E007E000000` | ~35% |
| 9 | `*` | `00183C7E18180000` | ~40% |
| 10 | `#` | `3C666E7666663C00` | ~50% |
| 11 | `%` | `66663C187E666600` | ~55% |
| 12 | `@` | `3C66667E60663C00` | ~60% |
| 13 | `8` | `3C66663C66663C00` | ~65% |
| 14 | `W` | `66666666667E6600` | ~70% |
| 15 | `█` | `FFFFFFFFFFFFFFFF` | 100% (full) |

## CRT Effects (Post-Processing)
From Efecto article — can be added as a third pass:
- **Scanlines**: `float scan = sin(vUv.y * resolution.y * 2.0) * 0.1;`
- **Curvature**: Barrel distortion on UV coordinates
- **Chromatic aberration**: Separate RGB sampling with slight offsets
- **Vignette**: `float vignette = 1.0 - length(uv - 0.5) * 2.0;`
- **Bloom**: Downsample → blur → upsample → add

## Performance Notes
- Compute shader: 16×16 workgroups, one invocation per pixel
- Atlas sampling is fast (nearest filter, single texture)
- Luma sampling at cell center avoids expensive box filter
- Strength parameter allows smooth fade (0.0 = original image)
- Use `rgba16float` for HDR headroom in post-processing

## References
- **bdip-core WGSL source**: https://docs.rs/crate/bdip-core/latest/source/shaders-wip/shaders/ascii_art/ascii_art_ascii.wgsl
- **Efecto article**: https://tympanus.net/codrops/2026/01/04/efecto-building-real-time-ascii-and-dithering-effects-with-webgl-shaders/
- **Efecto live demo**: https://efecto.app/