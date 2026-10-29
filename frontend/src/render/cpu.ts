import { layoutCanvas } from "./layout";

/**
 * CPU / Canvas2D fallback renderer.
 *
 * Draws the worker's CPU reference raster (which already includes the
 * ASCII/original presentation blend). CRT effects are GPU-only: in fallback
 * mode they are disabled in the UI and identified with a visible note —
 * never silently claimed as rendered.
 */
export class CpuRenderer {
  readonly kind = "canvas2d" as const;
  private ctx: CanvasRenderingContext2D;
  private offscreen: HTMLCanvasElement;
  private offctx: CanvasRenderingContext2D;
  private lastRaster: Uint8Array | null = null;

  constructor(private canvas: HTMLCanvasElement) {
    const ctx = canvas.getContext("2d", { alpha: true });
    if (!ctx) throw new Error("Canvas2D unavailable");
    this.ctx = ctx;
    this.offscreen = document.createElement("canvas");
    const off = this.offscreen.getContext("2d", { alpha: true, willReadFrequently: false });
    if (!off) throw new Error("offscreen 2D unavailable");
    this.offctx = off;
    this.ctx.imageSmoothingEnabled = false;
  }

  static create(canvas: HTMLCanvasElement): CpuRenderer {
    return new CpuRenderer(canvas);
  }

  draw(raster: Uint8Array, width: number, height: number): void {
    this.lastRaster = raster;

    if (this.offscreen.width !== width || this.offscreen.height !== height) {
      this.offscreen.width = width;
      this.offscreen.height = height;
    }
    // Copy into a dedicated ArrayBuffer-backed view (TS requires ArrayBuffer,
    // and putImageData takes ownership semantics seriously).
    const image = new ImageData(new Uint8ClampedArray(raster), width, height);
    this.offctx.putImageData(image, 0, 0);

    const { cw, ch } = layoutCanvas(this.canvas, width, height);
    this.ctx.imageSmoothingEnabled = false;
    this.ctx.clearRect(0, 0, cw, ch);
    this.ctx.drawImage(this.offscreen, 0, 0, cw, ch);
  }

  /** Export source: the exact native-resolution blended raster. */
  exportNative(): Uint8Array {
    if (!this.lastRaster) throw new Error("nothing rendered yet");
    return this.lastRaster.slice();
  }

  dispose(): void {
    this.offscreen.width = 0;
    this.offscreen.height = 0;
    this.lastRaster = null;
  }
}
