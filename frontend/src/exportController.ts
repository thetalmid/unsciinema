/**
 * Export orchestration (main thread).
 *
 * - PNG / WebP take the renderer's native-resolution, effect-included
 *   readback (`exportNative()`), so exports show what the user sees (CRT +
 *   ASCII/original blend baked in at output resolution).
 * - Text / ANSI / HTML run in the worker against the current conversion;
 *   HTML embeds the real variant WOFF fetched same-origin.
 * - GIF encodes incrementally in the worker with cooperative cancellation.
 *
 * Buffers passed to the worker are freshly produced (readback/slice), so
 * transferring them never detaches preview state.
 */
import {
  setPhase,
  setGifProgress,
  setStatusMessage,
  setLastError,
  preview,
  sourceInfo,
} from "./state/app";
import { convertParams } from "./state/params";
import { WorkerClient } from "./worker/client";
import type { ParamsDTO, ResultDTO } from "./worker/protocol";
import { createSignal } from "solid-js";

const [webpQuality, setWebpQuality] = createSignal(90);
const [webpLossless, setWebpLossless] = createSignal(false);
export { webpQuality, setWebpQuality, webpLossless, setWebpLossless };

let client: WorkerClient | null = null;
let exportNative: (() => Uint8Array) | null = null;
let fontWoff: (() => Promise<Uint8Array>) | null = null;

export function bindExport(ctx: {
  client: WorkerClient;
  exportNative: () => Uint8Array;
  fontWoff: () => Promise<Uint8Array>;
}): void {
  client = ctx.client;
  exportNative = ctx.exportNative;
  fontWoff = ctx.fontWoff;
}

/** Trigger a browser download for produced bytes (local only, blob: URL). */
export function downloadBytes(name: string, mime: string, bytes: Uint8Array): void {
  const blob = new Blob([bytes as BlobPart], { type: mime });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = name;
  a.rel = "noopener";
  document.body.appendChild(a);
  a.click();
  a.remove();
  // Give the browser a beat to start the download before revoking.
  setTimeout(() => URL.revokeObjectURL(url), 30_000);
}

function requireClient(): WorkerClient {
  if (!client) throw new Error("export controller not bound");
  return client;
}

/** Snapshot the UI convert params as a worker-safe DTO (cloned buffers). */
export function currentParamsDto(): ParamsDTO {
  const p = convertParams();
  return {
    cellSize: p.cellSize,
    targetWidth: p.targetWidth,
    contrast: p.contrast,
    gamma: p.gamma,
    ditherAlgorithm: p.ditherAlgorithm,
    ditherStrength: p.ditherStrength,
    threshold: p.threshold,
    asciiStrength: p.asciiStrength,
    colorMode: p.colorMode,
    paletteIndex: p.paletteIndex,
    customPalette: p.customPalette ? Uint32Array.from(p.customPalette) : null,
    alphaMode: p.alphaMode,
    background: p.background,
    fontVariant: p.fontVariant,
  };
}

export function exportRaster(kind: "png" | "webp"): void {
  try {
    const snap = preview();
    if (!snap || !exportNative) throw new Error("nothing to export yet");
    const rgba = exportNative();
    if (rgba.length !== snap.result.outputWidth * snap.result.outputHeight * 4) {
      throw new Error("export buffer size mismatch");
    }
    setPhase("exporting");
    setLastError(null);
    setStatusMessage(`Encoding ${kind.toUpperCase()}…`);
    requireClient().requestExport({
      t: "exportRaster",
      kind,
      rgba,
      width: snap.result.outputWidth,
      height: snap.result.outputHeight,
      quality: webpQuality(),
      lossless: webpLossless(),
    });
  } catch (e) {
    fail(e);
  }
}

export function exportTextOrAnsi(kind: "text" | "ansi"): void {
  try {
    if (!preview()) throw new Error("nothing to export yet");
    setPhase("exporting");
    setLastError(null);
    setStatusMessage(kind === "text" ? "Encoding text…" : "Encoding ANSI…");
    requireClient().requestExport({ t: "exportText", kind });
  } catch (e) {
    fail(e);
  }
}

export async function exportHtml(): Promise<void> {
  try {
    const snap = preview();
    if (!snap || !fontWoff) throw new Error("nothing to export yet");
    setPhase("exporting");
    setLastError(null);
    setStatusMessage("Fetching variant WOFF + encoding HTML…");
    const woff = await fontWoff();
    // Colored per-cell HTML spans apply to color/palette modes only.
    const isColored = convertParams().colorMode !== 0;
    requireClient().requestExport({
      t: "exportHtml",
      woff,
      colored: isColored,
    });
  } catch (e) {
    fail(e);
  }
}

export function exportGif(): void {
  try {
    const info = sourceInfo();
    if (!info) throw new Error("nothing to export yet");
    setPhase("exporting");
    setLastError(null);
    setGifProgress({ done: 0, total: info.animated ? info.frameCount : 1 });
    setStatusMessage("Encoding GIF frame by frame (cancellable)…");
    requireClient().startGifExport(currentParamsDto(), !info.animated);
  } catch (e) {
    setGifProgress(null);
    fail(e);
  }
}

export function cancelGif(): void {
  requireClient().cancelActiveJob();
}

function fail(e: unknown): void {
  setPhase(preview() ? "ready" : "empty");
  setGifProgress(null);
  const msg = e instanceof Error ? e.message : String(e);
  setLastError(msg);
  setStatusMessage(`Export failed: ${msg}`);
}

/** Called from the worker callbacks in App. */
export function onExported(bytes: Uint8Array, name: string, mime: string): void {
  setGifProgress(null);
  setPhase(preview() ? "ready" : "empty");
  setStatusMessage(`Exported ${name} (${(bytes.length / 1024).toFixed(1)} KiB)`);
  downloadBytes(name, mime, bytes);
}

export function onExportError(message: string): void {
  fail(new Error(message));
}

export type { ResultDTO };
