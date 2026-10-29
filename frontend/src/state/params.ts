/**
 * UI parameter state (Solid 2 signals).
 *
 * Split into two signals so each effect tracks exactly what it needs:
 * - `convertParams`: anything the Rust pipeline consumes (reconvert on change);
 * - `crtParams`: renderer-only presentation (no reconvert — redraw only).
 *
 * Writes happen only in event handlers; reads in JSX/memos/tracking phase.
 * Signal writes are microtask-batched: after a setter in a handler, call
 * `flush()` only when a synchronous read is genuinely needed (load flow).
 */
import { createSignal } from "solid-js";

export interface ConvertParams {
  /** source pixels per cell (height); 1..=64 fractional allowed */
  cellSize: number;
  /** forced source width in px; 0 = keep aspect/no resize */
  targetWidth: number;
  /** 0.1..=3.0, 1.0 neutral */
  contrast: number;
  /** 0.5..=2.5, 1.0 neutral (pow(c, 1/gamma)) */
  gamma: number;
  /** DitherAlgorithm id 0..=10 (5 = Random Noise, 10 = Blue Noise) */
  ditherAlgorithm: number;
  /** 0..=1, 0 == no dithering */
  ditherStrength: number;
  /** 0..=1, midpoint 0.5 neutral */
  threshold: number;
  /** presentation blend: 0 = original, 1 = full ASCII (default) */
  asciiStrength: number;
  /** ColorMode 0 mono / 1 color / 2 palette */
  colorMode: number;
  /** preset index into wasm palettes; -1 = custom */
  paletteIndex: number;
  /** custom palette (0xRRGGBBAA, 1..=64 entries) */
  customPalette: Uint32Array | null;
  /** AlphaMode 0 ignore / 1 preserve / 2 blend-over-background */
  alphaMode: number;
  /** 0xRRGGBBAA */
  background: number;
  /** FontVariant 0..=6 */
  fontVariant: number;
}

export interface CrtParams {
  /** 0..=1 scanline strength (0 neutral) */
  scanlines: number;
  /** 0..=1 barrel curvature (0 neutral) */
  curvature: number;
  /** 0..=1 bloom strength (0 neutral) */
  bloom: number;
  /** 0..=1 vignette strength (0 neutral) */
  vignette: number;
}

export const CONVERT_DEFAULTS: ConvertParams = {
  cellSize: 8,
  targetWidth: 0,
  contrast: 1.0,
  gamma: 1.0,
  ditherAlgorithm: 0,
  ditherStrength: 1.0,
  threshold: 0.5,
  asciiStrength: 1.0,
  colorMode: 0,
  paletteIndex: 1, // "terminal"
  customPalette: null,
  alphaMode: 0,
  background: 0x000000ff,
  fontVariant: 0,
};

export const CRT_DEFAULTS: CrtParams = {
  scanlines: 0,
  curvature: 0,
  bloom: 0,
  vignette: 0,
};

const [convertParams, setConvertParams] = createSignal<ConvertParams>({
  ...CONVERT_DEFAULTS,
});
const [crtParams, setCrtParams] = createSignal<CrtParams>({
  ...CRT_DEFAULTS,
});

export { convertParams, crtParams };

export function setConvert<K extends keyof ConvertParams>(
  key: K,
  value: ConvertParams[K],
): void {
  setConvertParams((p) => ({ ...p, [key]: value }));
}

export function setCrt<K extends keyof CrtParams>(
  key: K,
  value: CrtParams[K],
): void {
  setCrtParams((p) => ({ ...p, [key]: value }));
}

export function resetAllParams(): void {
  setConvertParams({ ...CONVERT_DEFAULTS });
  setCrtParams({ ...CRT_DEFAULTS });
}

/** Clamp helper for numeric inputs (keeps values inside validated ranges). */
export function clamp(v: number, lo: number, hi: number): number {
  return Math.min(hi, Math.max(lo, v));
}
