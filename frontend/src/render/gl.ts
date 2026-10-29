/**
 * WebGL2 renderer (GLSL ES 3.00).
 *
 * Pipeline (all offscreen buffers at the NATIVE output resolution unless
 * noted):
 *   glyph pass  -> sceneFBO   (faithful raster_cells port; blend applied)
 *   bloom H/V   -> bloomFBOs  (half res; skipped entirely when bloom = 0)
 *   post pass   -> postFBO    (CRT effects; exact passthrough at 0)
 *   blit        -> canvas     (device-pixel backing, NEAREST)
 *
 * Exports read back `postFBO` (so PNG/WebP contain what the user sees,
 * CRT included) by flipping rows to restore the top-left contract origin.
 * readPixels/getError are NEVER called during live frames — only on export.
 *
 * Context loss: `webglcontextlost` is prevented, state is flagged and the
 * UI is notified; `webglcontextrestored` rebuilds every GPU resource and
 * redraws the last snapshot.
 */
import type { CrtParams } from "../state/params";
import type { ResultDTO } from "../worker/protocol";
import { layoutCanvas } from "./layout";
import {
  BLIT_FS,
  BLOOM_H_FS,
  BLOOM_V_FS,
  GLYPH_FS,
  POST_FS,
  QUAD_VS,
} from "./shaders";

export interface DrawInput {
  result: ResultDTO;
  /** Source resized to output dimensions (RGBA8, top-left, straight). */
  source: Uint8Array;
  /** Blended CPU reference raster (used by the Canvas2D fallback). */
  raster: Uint8Array;
  asciiStrength: number;
}

type UniformMap = Record<string, WebGLUniformLocation | null>;

function compile(gl: WebGL2RenderingContext, type: number, src: string): WebGLShader {
  const sh = gl.createShader(type);
  if (!sh) throw new Error("createShader failed");
  gl.shaderSource(sh, src);
  gl.compileShader(sh);
  if (!gl.getShaderParameter(sh, gl.COMPILE_STATUS)) {
    const log = gl.getShaderInfoLog(sh) ?? "unknown";
    gl.deleteShader(sh);
    throw new Error(`shader compile failed: ${log}`);
  }
  return sh;
}

function link(
  gl: WebGL2RenderingContext,
  vsSrc: string,
  fsSrc: string,
): WebGLProgram {
  const vs = compile(gl, gl.VERTEX_SHADER, vsSrc);
  const fs = compile(gl, gl.FRAGMENT_SHADER, fsSrc);
  const prog = gl.createProgram();
  if (!prog) throw new Error("createProgram failed");
  gl.attachShader(prog, vs);
  gl.attachShader(prog, fs);
  gl.bindAttribLocation(prog, 0, "a_pos");
  gl.linkProgram(prog);
  gl.deleteShader(vs);
  gl.deleteShader(fs);
  if (!gl.getProgramParameter(prog, gl.LINK_STATUS)) {
    const log = gl.getProgramInfoLog(prog) ?? "unknown";
    gl.deleteProgram(prog);
    throw new Error(`program link failed: ${log}`);
  }
  return prog;
}

function uniforms(gl: WebGL2RenderingContext, prog: WebGLProgram, names: string[]): UniformMap {
  const out: UniformMap = {};
  for (const n of names) out[n] = gl.getUniformLocation(prog, n);
  return out;
}

interface TexPair {
  tex: WebGLTexture;
  fb: WebGLFramebuffer;
  w: number;
  h: number;
}

export class GlRenderer {
  readonly kind = "webgl2" as const;
  private gl: WebGL2RenderingContext;
  private quad: WebGLBuffer;
  private progGlyph: WebGLProgram;
  private progBloomH: WebGLProgram;
  private progBloomV: WebGLProgram;
  private progPost: WebGLProgram;
  private progBlit: WebGLProgram;
  private uGlyph: UniformMap;
  private uBloomH: UniformMap;
  private uBloomV: UniformMap;
  private uPost: UniformMap;
  private uBlit: UniformMap;

  private texGlyph: WebGLTexture | null = null;
  private texAtlas: WebGLTexture | null = null;
  private texFg: WebGLTexture | null = null;
  private texBg: WebGLTexture | null = null;
  private texSrc: WebGLTexture | null = null;
  private texDummy: WebGLTexture;
  private scene: TexPair | null = null;
  private post: TexPair | null = null;
  private bloomA: TexPair | null = null;
  private bloomB: TexPair | null = null;
  private nearestSampler: WebGLSampler;
  private linearSampler: WebGLSampler;

  private lastInput: DrawInput | null = null;
  private lastUploadVersion = -1;
  private crt: CrtParams = {
    scanlines: 0,
    curvature: 0,
    bloom: 0,
    vignette: 0,
  };
  private lost = false;
  private disposed = false;
  private onNote: (note: string | null) => void;
  private ctxLostHandler: (e: Event) => void;
  private ctxRestoredHandler: () => void;

  private constructor(
    private canvas: HTMLCanvasElement,
    onNote: (note: string | null) => void,
  ) {
    this.onNote = onNote;
    const gl = canvas.getContext("webgl2", {
      alpha: true,
      premultipliedAlpha: false,
      antialias: false,
      depth: false,
      stencil: false,
      preserveDrawingBuffer: false,
      powerPreference: "high-performance",
    });
    if (!gl) throw new Error("WebGL2 unavailable");
    this.gl = gl;

    this.progGlyph = link(gl, QUAD_VS, GLYPH_FS);
    this.progBloomH = link(gl, QUAD_VS, BLOOM_H_FS);
    this.progBloomV = link(gl, QUAD_VS, BLOOM_V_FS);
    this.progPost = link(gl, QUAD_VS, POST_FS);
    this.progBlit = link(gl, QUAD_VS, BLIT_FS);
    this.uGlyph = uniforms(gl, this.progGlyph, [
      "u_outSize", "u_glyphW", "u_glyphH", "u_glyphTex", "u_atlasTex",
      "u_fgTex", "u_bgTex", "u_srcTex", "u_asciiStrength",
    ]);
    this.uBloomH = uniforms(gl, this.progBloomH, ["u_scene", "u_texel", "u_threshold"]);
    this.uBloomV = uniforms(gl, this.progBloomV, ["u_input", "u_texel"]);
    this.uPost = uniforms(gl, this.progPost, [
      "u_scene", "u_bloom", "u_scanlines", "u_curvature", "u_bloomAmt", "u_vignette",
    ]);
    this.uBlit = uniforms(gl, this.progBlit, ["u_post"]);

    const quad = gl.createBuffer();
    if (!quad) throw new Error("createBuffer failed");
    this.quad = quad;
    gl.bindBuffer(gl.ARRAY_BUFFER, quad);
    gl.bufferData(
      gl.ARRAY_BUFFER,
      new Float32Array([-1, -1, 1, -1, -1, 1, 1, 1]),
      gl.STATIC_DRAW,
    );
    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 0, 0);

    this.nearestSampler = gl.createSampler()!;
    gl.samplerParameteri(this.nearestSampler, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.samplerParameteri(this.nearestSampler, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    gl.samplerParameteri(this.nearestSampler, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.samplerParameteri(this.nearestSampler, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    this.linearSampler = gl.createSampler()!;
    gl.samplerParameteri(this.linearSampler, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    gl.samplerParameteri(this.linearSampler, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
    gl.samplerParameteri(this.linearSampler, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.samplerParameteri(this.linearSampler, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);

    // 1x1 dummy keeps samplers complete when bloom is skipped.
    const dummy = gl.createTexture();
    if (!dummy) throw new Error("createTexture failed");
    this.texDummy = dummy;
    gl.bindTexture(gl.TEXTURE_2D, dummy);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA8, 1, 1, 0, gl.RGBA, gl.UNSIGNED_BYTE,
      new Uint8Array([0, 0, 0, 0]));

    gl.disable(gl.DEPTH_TEST);
    gl.disable(gl.BLEND);
    gl.disable(gl.CULL_FACE);
    gl.clearColor(0, 0, 0, 0);

    this.ctxLostHandler = (e) => {
      e.preventDefault();
      this.lost = true;
      this.onNote("WebGL context lost — waiting for restore…");
    };
    this.ctxRestoredHandler = () => {
      this.lost = false;
      try {
        this.rebuildAfterRestore();
        this.onNote("WebGL context restored.");
        if (this.lastInput) this.drawFrame(this.lastInput);
        setTimeout(() => this.onNote(null), 2500);
      } catch (e) {
        this.onNote(`WebGL restore failed: ${errText(e)} — using CPU preview.`);
      }
    };
    canvas.addEventListener("webglcontextlost", this.ctxLostHandler);
    canvas.addEventListener("webglcontextrestored", this.ctxRestoredHandler);
  }

  static create(
    canvas: HTMLCanvasElement,
    onNote: (note: string | null) => void,
  ): GlRenderer {
    return new GlRenderer(canvas, onNote);
  }

  setParams(crt: CrtParams): void {
    this.crt = crt;
  }

  // ---- resource management --------------------------------------------------

  private ensureTextures(r: ResultDTO): void {
    const needW = r.outputWidth;
    const needH = r.outputHeight;
    const rebuild =
      !this.scene || this.scene.w !== needW || this.scene.h !== needH;
    if (rebuild) {
      this.destroyTargets();
      this.scene = this.makeTarget(needW, needH);
      this.post = this.makeTarget(needW, needH);
      const bw = Math.max(1, Math.ceil(needW / 2));
      const bh = Math.max(1, Math.ceil(needH / 2));
      this.bloomA = this.makeTarget(bw, bh);
      this.bloomB = this.makeTarget(bw, bh);
      this.lastUploadVersion = -1;
    }
  }

  private makeTarget(w: number, h: number): TexPair {
    const gl = this.gl;
    const tex = gl.createTexture()!;
    gl.bindTexture(gl.TEXTURE_2D, tex);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA8, w, h, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    const fb = gl.createFramebuffer()!;
    gl.bindFramebuffer(gl.FRAMEBUFFER, fb);
    gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, tex, 0);
    const status = gl.checkFramebufferStatus(gl.FRAMEBUFFER);
    gl.bindFramebuffer(gl.FRAMEBUFFER, null);
    if (status !== gl.FRAMEBUFFER_COMPLETE) {
      throw new Error(`framebuffer incomplete (0x${status.toString(16)})`);
    }
    return { tex, fb, w, h };
  }

  private destroyTargets(): void {
    const gl = this.gl;
    for (const t of [this.scene, this.post, this.bloomA, this.bloomB]) {
      if (t) {
        gl.deleteTexture(t.tex);
        gl.deleteFramebuffer(t.fb);
      }
    }
    this.scene = null;
    this.post = null;
    this.bloomA = null;
    this.bloomB = null;
    this.lastUploadVersion = -1;
  }

  private uploadInputs(r: ResultDTO): void {
    const gl = this.gl;
    // glyph indices: cols x rows R32UI (Uint32 view of Uint16 indices).
    const cells = r.cols * r.rows;
    const glyphU32 = new Uint32Array(cells);
    glyphU32.set(r.glyphIndices);
    this.texGlyph = this.intTex(this.texGlyph, gl.R32UI, gl.RED_INTEGER,
      gl.UNSIGNED_INT, r.cols, r.rows, glyphU32) ?? this.texGlyph;

    // atlas coverage: R8UI from atlas RGBA R channel.
    const ink = new Uint8Array(r.atlasWidth * r.atlasHeight);
    for (let i = 0; i < ink.length; i++) ink[i] = r.atlasRGBA[i * 4];
    this.texAtlas = this.intTex(this.texAtlas, gl.R8UI, gl.RED_INTEGER,
      gl.UNSIGNED_BYTE, r.atlasWidth, r.atlasHeight, ink) ?? this.texAtlas;

    // fg/bg packed with explicit shifts (0xRRGGBBAA) — never reinterpreted.
    const fg = new Uint32Array(cells);
    const bg = new Uint32Array(cells);
    const f8 = r.cellForegroundRGBA;
    const b8 = r.cellBackgroundRGBA;
    for (let i = 0; i < cells; i++) {
      const o = i * 4;
      fg[i] =
        ((f8[o] << 24) | (f8[o + 1] << 16) | (f8[o + 2] << 8) | f8[o + 3]) >>> 0;
      bg[i] =
        ((b8[o] << 24) | (b8[o + 1] << 16) | (b8[o + 2] << 8) | b8[o + 3]) >>> 0;
    }
    this.texFg = this.intTex(this.texFg, gl.R32UI, gl.RED_INTEGER,
      gl.UNSIGNED_INT, r.cols, r.rows, fg) ?? this.texFg;
    this.texBg = this.intTex(this.texBg, gl.R32UI, gl.RED_INTEGER,
      gl.UNSIGNED_INT, r.cols, r.rows, bg) ?? this.texBg;
  }

  private intTex(
    existing: WebGLTexture | null,
    internal: number,
    format: number,
    type: number,
    w: number,
    h: number,
    data: Uint8Array | Uint32Array,
  ): WebGLTexture | null {
    const gl = this.gl;
    let tex = existing;
    if (tex) {
      gl.deleteTexture(tex);
      tex = null;
    }
    tex = gl.createTexture();
    if (!tex) return null;
    gl.bindTexture(gl.TEXTURE_2D, tex);
    gl.pixelStorei(gl.UNPACK_ALIGNMENT, 1);
    gl.texImage2D(gl.TEXTURE_2D, 0, internal, w, h, 0, format, type, data as never);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    return tex;
  }

  private uploadSource(input: DrawInput): void {
    const gl = this.gl;
    const { result, source } = input;
    if (this.texSrc) gl.deleteTexture(this.texSrc);
    const tex = gl.createTexture();
    if (!tex) throw new Error("src texture failed");
    gl.bindTexture(gl.TEXTURE_2D, tex);
    gl.pixelStorei(gl.UNPACK_ALIGNMENT, 1);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA8, result.outputWidth,
      result.outputHeight, 0, gl.RGBA, gl.UNSIGNED_BYTE, source);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    this.texSrc = tex;
  }

  private bindUnit(unit: number, tex: WebGLTexture | null, sampler: WebGLSampler): void {
    const gl = this.gl;
    gl.activeTexture(gl.TEXTURE0 + unit);
    gl.bindTexture(gl.TEXTURE_2D, tex ?? this.texDummy);
    gl.bindSampler(unit, sampler);
  }

  private drawQuad(): void {
    this.gl.drawArrays(this.gl.TRIANGLE_STRIP, 0, 4);
  }

  // ---- frame rendering ------------------------------------------------------

  drawFrame(input: DrawInput): void {
    if (this.disposed) return;
    if (this.lost) return; // draws resume after 'webglcontextrestored'
    try {
      this.ensureTextures(input.result);
      // Upload cell/atlas data whenever the snapshot (or GPU state) changed.
      if (this.lastInput !== input || this.lastUploadVersion < 0) {
        this.uploadInputs(input.result);
        this.lastUploadVersion = 1;
      }
      this.uploadSource(input);
      this.lastInput = input;
      this.renderChain();
    } catch (e) {
      this.onNote(`GL draw failed: ${errText(e)}`);
      throw e;
    }
  }

  private renderChain(): void {
    const gl = this.gl;
    const input = this.lastInput;
    if (!input || !this.scene || !this.post || !this.bloomA || !this.bloomB) return;
    const r = input.result;

    // Deterministic fit of native output into the pane (CSS px x DPR backing).
    const { cw, ch } = layoutCanvas(this.canvas, r.outputWidth, r.outputHeight);

    // 1) glyph pass -> scene FBO (native res)
    gl.bindFramebuffer(gl.FRAMEBUFFER, this.scene.fb);
    gl.viewport(0, 0, this.scene.w, this.scene.h);
    gl.useProgram(this.progGlyph);
    gl.uniform2i(this.uGlyph.u_outSize!, r.outputWidth, r.outputHeight);
    gl.uniform1i(this.uGlyph.u_glyphW!, r.glyphWidth);
    gl.uniform1i(this.uGlyph.u_glyphH!, r.glyphHeight);
    gl.uniform1f(this.uGlyph.u_asciiStrength!, input.asciiStrength);
    this.bindUnit(0, this.texGlyph, this.nearestSampler);
    this.bindUnit(1, this.texAtlas, this.nearestSampler);
    this.bindUnit(2, this.texFg, this.nearestSampler);
    this.bindUnit(3, this.texBg, this.nearestSampler);
    this.bindUnit(4, this.texSrc, this.nearestSampler);
    gl.uniform1i(this.uGlyph.u_glyphTex!, 0);
    gl.uniform1i(this.uGlyph.u_atlasTex!, 1);
    gl.uniform1i(this.uGlyph.u_fgTex!, 2);
    gl.uniform1i(this.uGlyph.u_bgTex!, 3);
    gl.uniform1i(this.uGlyph.u_srcTex!, 4);
    this.drawQuad();

    // 2) bloom (skipped when 0 — no extra passes, no stray reads)
    if (this.crt.bloom > 0) {
      gl.bindFramebuffer(gl.FRAMEBUFFER, this.bloomA.fb);
      gl.viewport(0, 0, this.bloomA.w, this.bloomA.h);
      gl.useProgram(this.progBloomH);
      this.bindUnit(0, this.scene.tex, this.linearSampler);
      gl.uniform1i(this.uBloomH.u_scene!, 0);
      gl.uniform2f(this.uBloomH.u_texel!, 1 / this.scene.w, 1 / this.scene.h);
      gl.uniform1f(this.uBloomH.u_threshold!, 0.5);
      this.drawQuad();

      gl.bindFramebuffer(gl.FRAMEBUFFER, this.bloomB.fb);
      gl.viewport(0, 0, this.bloomB.w, this.bloomB.h);
      gl.useProgram(this.progBloomV);
      this.bindUnit(0, this.bloomA.tex, this.linearSampler);
      gl.uniform1i(this.uBloomV.u_input!, 0);
      gl.uniform2f(this.uBloomV.u_texel!, 1 / this.bloomA.w, 1 / this.bloomA.h);
      this.drawQuad();
    }

    // 3) post -> post FBO (native res; export source)
    gl.bindFramebuffer(gl.FRAMEBUFFER, this.post.fb);
    gl.viewport(0, 0, this.post.w, this.post.h);
    gl.useProgram(this.progPost);
    this.bindUnit(0, this.scene.tex, this.nearestSampler);
    this.bindUnit(1, this.crt.bloom > 0 ? this.bloomB.tex : this.texDummy, this.linearSampler);
    gl.uniform1i(this.uPost.u_scene!, 0);
    gl.uniform1i(this.uPost.u_bloom!, 1);
    gl.uniform1f(this.uPost.u_scanlines!, this.crt.scanlines);
    gl.uniform1f(this.uPost.u_curvature!, this.crt.curvature);
    gl.uniform1f(this.uPost.u_bloomAmt!, this.crt.bloom);
    gl.uniform1f(this.uPost.u_vignette!, this.crt.vignette);
    this.drawQuad();

    // 4) blit -> canvas
    gl.bindFramebuffer(gl.FRAMEBUFFER, null);
    gl.viewport(0, 0, cw, ch);
    gl.useProgram(this.progBlit);
    this.bindUnit(0, this.post.tex, this.nearestSampler);
    gl.uniform1i(this.uBlit.u_post!, 0);
    this.drawQuad();
  }

  /** Read back the native-res postprocessed frame (export time only). */
  exportNative(): Uint8Array {
    if (this.disposed) throw new Error("renderer disposed");
    if (this.lost) throw new Error("WebGL context is lost");
    if (!this.lastInput) throw new Error("nothing rendered yet");
    const gl = this.gl;
    // Re-render the chain so readback reflects the current state.
    this.renderChain();
    const post = this.post;
    if (!post) throw new Error("no post framebuffer");
    const pixels = new Uint8Array(post.w * post.h * 4);
    gl.bindFramebuffer(gl.FRAMEBUFFER, post.fb);
    gl.readPixels(0, 0, post.w, post.h, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
    gl.bindFramebuffer(gl.FRAMEBUFFER, null);
    // GL origin is bottom-left; flip rows back to top-left (contract).
    const flipped = new Uint8Array(pixels.length);
    const stride = post.w * 4;
    for (let y = 0; y < post.h; y++) {
      const src = (post.h - 1 - y) * stride;
      flipped.set(pixels.subarray(src, src + stride), y * stride);
    }
    return flipped;
  }

  // ---- context loss / disposal --------------------------------------------

  private rebuildAfterRestore(): void {
    // After restore all objects are lost: recreate programs and targets.
    const gl = this.gl;
    this.progGlyph = link(gl, QUAD_VS, GLYPH_FS);
    this.progBloomH = link(gl, QUAD_VS, BLOOM_H_FS);
    this.progBloomV = link(gl, QUAD_VS, BLOOM_V_FS);
    this.progPost = link(gl, QUAD_VS, POST_FS);
    this.progBlit = link(gl, QUAD_VS, BLIT_FS);
    this.uGlyph = uniforms(gl, this.progGlyph, [
      "u_outSize", "u_glyphW", "u_glyphH", "u_glyphTex", "u_atlasTex",
      "u_fgTex", "u_bgTex", "u_srcTex", "u_asciiStrength",
    ]);
    this.uBloomH = uniforms(gl, this.progBloomH, ["u_scene", "u_texel", "u_threshold"]);
    this.uBloomV = uniforms(gl, this.progBloomV, ["u_input", "u_texel"]);
    this.uPost = uniforms(gl, this.progPost, [
      "u_scene", "u_bloom", "u_scanlines", "u_curvature", "u_bloomAmt", "u_vignette",
    ]);
    this.uBlit = uniforms(gl, this.progBlit, ["u_post"]);
    this.quad = gl.createBuffer()!;
    gl.bindBuffer(gl.ARRAY_BUFFER, this.quad);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1, -1, 1, -1, -1, 1, 1, 1]), gl.STATIC_DRAW);
    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 0, 0);
    this.nearestSampler = gl.createSampler()!;
    gl.samplerParameteri(this.nearestSampler, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.samplerParameteri(this.nearestSampler, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    gl.samplerParameteri(this.nearestSampler, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.samplerParameteri(this.nearestSampler, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    this.linearSampler = gl.createSampler()!;
    gl.samplerParameteri(this.linearSampler, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    gl.samplerParameteri(this.linearSampler, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
    gl.samplerParameteri(this.linearSampler, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.samplerParameteri(this.linearSampler, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    this.texDummy = gl.createTexture()!;
    gl.bindTexture(gl.TEXTURE_2D, this.texDummy);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA8, 1, 1, 0, gl.RGBA, gl.UNSIGNED_BYTE,
      new Uint8Array([0, 0, 0, 0]));
    this.texGlyph = null;
    this.texAtlas = null;
    this.texFg = null;
    this.texBg = null;
    this.texSrc = null;
    this.destroyTargets();
    // Force full re-upload of cell data on the next draw.
    const li = this.lastInput;
    this.lastInput = null;
    if (li) this.uploadInputs(li.result);
    this.lastInput = li;
    this.lastUploadVersion = -1;
  }

  dispose(): void {
    if (this.disposed) return;
    this.disposed = true;
    const gl = this.gl;
    this.canvas.removeEventListener("webglcontextlost", this.ctxLostHandler);
    this.canvas.removeEventListener("webglcontextrestored", this.ctxRestoredHandler);
    this.destroyTargets();
    for (const t of [this.texGlyph, this.texAtlas, this.texFg, this.texBg, this.texSrc, this.texDummy]) {
      if (t) gl.deleteTexture(t);
    }
    gl.deleteBuffer(this.quad);
    gl.deleteSampler(this.nearestSampler);
    gl.deleteSampler(this.linearSampler);
    for (const p of [this.progGlyph, this.progBloomH, this.progBloomV, this.progPost, this.progBlit]) {
      gl.deleteProgram(p);
    }
  }
}

function errText(e: unknown): string {
  if (e instanceof Error) return e.message;
  return String(e);
}
