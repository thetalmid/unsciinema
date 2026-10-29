/**
 * Renderer manager: prefers WebGL2, falls back to Canvas2D, exposes a single
 * `exportNative()` used by PNG/WebP exports (GL reads the post-processed
 * native-res FBO; CPU returns its reference raster).
 *
 * Mode changes are pushed back through callbacks so the UI can badge the
 * active renderer and disable/identify CRT controls in fallback.
 */
import type { CrtParams } from "../state/params";
import { CpuRenderer } from "./cpu";
import { GlRenderer, type DrawInput } from "./gl";

export type Mode = "webgl2" | "canvas2d";

export interface RendererEvents {
  onMode(mode: Mode, note: string | null): void;
}

export class PreviewManager {
  private gl: GlRenderer | null = null;
  private cpu: CpuRenderer | null = null;
  private mode: Mode | null = null;
  private note: string | null = null;
  private lastInput: DrawInput | null = null;
  private disposed = false;

  constructor(
    private canvas: HTMLCanvasElement,
    private events: RendererEvents,
  ) {
    this.initGl();
  }

  private setMode(mode: Mode, note: string | null): void {
    this.mode = mode;
    this.note = note;
    this.events.onMode(mode, note);
  }

  private initGl(): void {
    try {
      const gl = GlRenderer.create(this.canvas, (n) => {
        this.note = n;
        this.events.onMode(this.mode ?? "webgl2", n);
      });
      this.gl = gl;
      this.cpu = null;
      this.setMode("webgl2", null);
    } catch (e) {
      this.gl = null;
      const reason = e instanceof Error ? e.message : String(e);
      this.setMode("canvas2d", `WebGL2 unavailable (${reason}) — CPU preview.`);
      this.initCpu();
    }
  }

  private initCpu(): void {
    if (this.cpu) return;
    try {
      this.cpu = CpuRenderer.create(this.canvas);
    } catch (e) {
      const reason = e instanceof Error ? e.message : String(e);
      this.events.onMode("canvas2d", `CPU renderer failed: ${reason}`);
    }
  }

  get activeMode(): Mode | null {
    return this.mode;
  }

  get currentNote(): string | null {
    return this.note;
  }

  setParams(crt: CrtParams): void {
    this.gl?.setParams(crt);
    // CRT changes are draw-only (no reconvert). Redraw if we have content.
    if (this.lastInput) this.redrawOnly();
  }

  /** Push a new snapshot into the active renderer. */
  draw(input: DrawInput): void {
    if (this.disposed) return;
    this.lastInput = input;
    if (this.mode === "webgl2" && this.gl) {
      try {
        this.gl.drawFrame(input);
        return;
      } catch (e) {
        // Degrade visibly rather than showing a stale frame forever.
        const reason = e instanceof Error ? e.message : String(e);
        this.gl.dispose();
        this.gl = null;
        this.initCpu();
        this.setMode(
          "canvas2d",
          `WebGL2 failed (${reason}) — CPU preview (CRT effects off).`,
        );
      }
    }
    // The worker pre-blended the ASCII raster with the source at
    // asciiStrength — the CPU path draws exactly that (parity by reuse).
    this.cpu?.draw(
      input.raster,
      input.result.outputWidth,
      input.result.outputHeight,
    );
  }

  redrawOnly(): void {
    if (this.lastInput) this.draw(this.lastInput);
  }

  /** Native-resolution, effect-included export source. */
  exportNative(): Uint8Array {
    if (this.mode === "webgl2" && this.gl) return this.gl.exportNative();
    if (this.cpu) return this.cpu.exportNative();
    throw new Error("no renderer available");
  }

  dispose(): void {
    this.disposed = true;
    this.gl?.dispose();
    this.cpu?.dispose();
    this.gl = null;
    this.cpu = null;
    this.lastInput = null;
  }
}
