/**
 * Main <-> worker message protocol (structured-clone safe; typed arrays are
 * transferred, not copied, where possible).
 *
 * Generation rules:
 * - every convert request carries an `id` from a monotonic counter;
 * - the main thread discards responses whose id is older than the latest
 *   request (stale results never touch the preview);
 * - long loops (GIF export) check the cooperative `cancel` flag between
 *   frames; if the worker does not acknowledge promptly the main thread
 *   terminates and restarts it (hard cancellation) and resends the source.
 */

/** Plain, cloneable mirror of the UI's conversion params. */
export interface ParamsDTO {
  cellSize: number;
  targetWidth: number;
  contrast: number;
  gamma: number;
  ditherAlgorithm: number;
  ditherStrength: number;
  threshold: number;
  /** ASCII/original presentation blend: 0 = original, 1 = full ASCII. */
  asciiStrength: number;
  colorMode: number;
  /** Index into the 31 wasm-provided presets; -1 = customPalette. */
  paletteIndex: number;
  /** Custom palette (1..=64 entries, 0xRRGGBBAA); null = use preset. */
  customPalette: Uint32Array | null;
  alphaMode: number;
  /** 0xRRGGBBAA background. */
  background: number;
  fontVariant: number;
}

/** Plain mirror of the wasm ConversionResult (camelCase contract). */
export interface ResultDTO {
  cols: number;
  rows: number;
  glyphWidth: number;
  glyphHeight: number;
  outputWidth: number;
  outputHeight: number;
  glyphIndices: Uint16Array;
  charRamp: string[];
  cellForegroundRGBA: Uint8Array;
  cellBackgroundRGBA: Uint8Array;
  atlasRGBA: Uint8Array;
  atlasWidth: number;
  atlasHeight: number;
  glyphCount: number;
}

/** Plain mirror of the wasm SourceInfo. */
export interface SourceInfoDTO {
  width: number;
  height: number;
  frameCount: number;
  animated: boolean;
  format: string;
  delaysMs: Uint32Array;
  /** -1 = extension absent, 0 = loop forever, n>0 = repeat n times. */
  loopCode: number;
}

export type ExportKind = "png" | "webp" | "text" | "ansi" | "html" | "gif";

export type ToWorker =
  | { t: "load"; id: number; bytes: Uint8Array }
  | { t: "convert"; id: number; frame: number; params: ParamsDTO }
  | { t: "cancel"; id: number }
  | { t: "gifExport"; id: number; params: ParamsDTO; still: boolean }
  | {
      t: "exportText";
      id: number;
      kind: "text" | "ansi";
    }
  | {
      t: "exportHtml";
      id: number;
      woff: Uint8Array;
      colored: boolean;
    }
  | {
      t: "exportRaster";
      id: number;
      kind: "png" | "webp";
      rgba: Uint8Array;
      width: number;
      height: number;
      quality?: number;
      lossless?: boolean;
    };

/** Worker export requests without the client-assigned id (distributed Omit). */
export type ExportRequest =
  | Omit<Extract<ToWorker, { t: "exportText" }>, "id">
  | Omit<Extract<ToWorker, { t: "exportHtml" }>, "id">
  | Omit<Extract<ToWorker, { t: "exportRaster" }>, "id">;

export type FromWorker =
  | { t: "loaded"; id: number; info: SourceInfoDTO }
  | {
      t: "result";
      id: number;
      frame: number;
      result: ResultDTO;
      /** CPU reference raster with asciiStrength already applied. */
      raster: Uint8Array;
      /** Immutable source resized to output dimensions (renderer texture). */
      source: Uint8Array;
      /** Wall time of the Rust conversion (ms). */
      ms: number;
      /** asciiStrength used for this snapshot (renderer consistency). */
      asciiStrength: number;
    }
  | { t: "progress"; id: number; done: number; total: number }
  | {
      t: "exported";
      id: number;
      kind: ExportKind;
      name: string;
      mime: string;
      bytes: Uint8Array;
    }
  | { t: "cancelled"; id: number }
  | { t: "error"; id: number; message: string };
