/**
 * Deterministic canvas layout: fits the native output aspect into the
 * available pane at CSS size, then sets the backing store to CSS x DPR
 * (capped). Export paths never depend on this — they read the native
 * resolution FBO / CPU raster — so display rounding cannot leak into files.
 */
export interface CanvasLayout {
  cw: number;
  ch: number;
  dpr: number;
}

export function layoutCanvas(
  canvas: HTMLCanvasElement,
  outW: number,
  outH: number,
): CanvasLayout {
  const wrap = canvas.parentElement;
  const maxW = Math.max(64, (wrap?.clientWidth ?? 640) - 8);
  const maxH = Math.max(
    64,
    Math.min(Math.round(window.innerHeight * 0.62), (wrap?.clientHeight ?? 480) - 8),
  );
  const scale = Math.min(maxW / outW, maxH / outH);
  const cssW = Math.max(1, Math.floor(outW * scale));
  const cssH = Math.max(1, Math.floor(outH * scale));
  canvas.style.width = `${cssW}px`;
  canvas.style.height = `${cssH}px`;
  const dpr = Math.min(window.devicePixelRatio || 1, 4);
  const cw = Math.max(1, Math.round(cssW * dpr));
  const ch = Math.max(1, Math.round(cssH * dpr));
  if (canvas.width !== cw || canvas.height !== ch) {
    canvas.width = cw;
    canvas.height = ch;
  }
  return { cw, ch, dpr };
}
