/**
 * Converter worker (module worker, own execution context).
 *
 * - Initializes the wasm package exactly once for this context (top-level).
 * - Owns the single `UnsciiConverter` instance: `load` decodes the immutable
 *   source ONCE (all later converts reuse it — no repeated filtering/decoding).
 * - Convert responses are addressed by generation id; the main thread drops
 *   stale ones.
 * - Long GIF loops are cooperatively cancellable: they yield to the event
 *   loop between frames so `cancel` messages are actually processed.
 * - WebP encoding runs here via @jsquash/webp (Apache-2.0), with both the
 *   SIMD and fallback codec wasm files wired through `locateFile` so Vite
 *   emits them as base-aware assets.
 */
import initWasm, {
  ConversionResult,
  ConvertParams as WasmConvertParams,
  GifWriter,
  UnsciiConverter,
  palettePresetsJson,
} from "../generated/wasm/unsciinema_core.js";
import type {
  FromWorker,
  ParamsDTO,
  ResultDTO,
  SourceInfoDTO,
  ToWorker,
} from "./protocol";

// ---- initialization (once per context) ------------------------------------

await initWasm({
  module_or_path: new URL(
    "../generated/wasm/unsciinema_core_bg.wasm",
    import.meta.url,
  ),
});

const PALETTES = JSON.parse(palettePresetsJson()) as {
  name: string;
  colors: number[];
}[];

const conv = new UnsciiConverter();
let currentInfo: SourceInfoDTO | null = null;
/** Latest conversion kept alive for text/HTML/ANSI exports. */
let lastResult: ConversionResult | null = null;
const cancelIds = new Set<number>();

/** Typed worker scope (avoids DOM `Window` typing on `self`). */
const ctx = self as unknown as {
  postMessage(m: unknown, t?: Transferable[]): void;
  onmessage: ((e: MessageEvent<ToWorker>) => void) | null;
};

function post(msg: FromWorker, transfer: Transferable[] = []): void {
  ctx.postMessage(msg, transfer);
}

function errText(e: unknown): string {
  if (typeof e === "string") return e;
  if (e instanceof Error) return e.message;
  return String(e);
}

function yieldToEventLoop(): Promise<void> {
  // Lets queued `cancel` messages run between GIF frames (cooperative cancel).
  return new Promise((resolve) => setTimeout(resolve, 0));
}

/** Build a fresh wasm ConvertParams from the DTO (validated again in Rust). */
function buildParams(dto: ParamsDTO): WasmConvertParams {
  const p = new WasmConvertParams();
  p.cellSize = dto.cellSize;
  p.targetWidth = dto.targetWidth;
  p.contrast = dto.contrast;
  p.gamma = dto.gamma;
  p.ditherAlgorithm = dto.ditherAlgorithm;
  p.ditherStrength = dto.ditherStrength;
  p.threshold = dto.threshold;
  p.colorMode = dto.colorMode;
  p.alphaMode = dto.alphaMode;
  p.background = dto.background;
  p.fontVariant = dto.fontVariant;
  // Palette: always set something valid (1..=64) so `setPalette` validates;
  // non-palette color modes ignore it inside the Rust pipeline.
  if (dto.colorMode === 2 && dto.customPalette && dto.customPalette.length > 0) {
    p.setPalette(dto.customPalette);
  } else if (dto.paletteIndex >= 0 && dto.paletteIndex < PALETTES.length) {
    p.setPalette(new Uint32Array(PALETTES[dto.paletteIndex].colors));
  } else {
    p.setPalette(new Uint32Array(PALETTES[1].colors));
  }
  return p;
}

/** Copy a wasm ConversionResult into a plain DTO (typed arrays become the
 * transferable payloads). The wasm object itself is kept or freed by callers. */
function toDTO(r: ConversionResult): ResultDTO {
  return {
    cols: r.cols,
    rows: r.rows,
    glyphWidth: r.glyphWidth,
    glyphHeight: r.glyphHeight,
    outputWidth: r.outputWidth,
    outputHeight: r.outputHeight,
    glyphIndices: r.glyphIndices, // getter clone (fresh buffer)
    charRamp: r.charRamp,
    cellForegroundRGBA: r.cellForegroundRGBA,
    cellBackgroundRGBA: r.cellBackgroundRGBA,
    atlasRGBA: r.atlasRGBA,
    atlasWidth: r.atlasWidth,
    atlasHeight: r.atlasHeight,
    glyphCount: r.glyphCount,
  };
}

// ---- WebP (@jsquash/webp, Apache-2.0) -------------------------------------

import webpEncode, { init as webpInit } from "@jsquash/webp/encode";
import webpWasmFallback from "@jsquash/webp/codec/enc/webp_enc.wasm?url";
import webpWasmSimd from "@jsquash/webp/codec/enc/webp_enc_simd.wasm?url";

let webpReady: Promise<void> | null = null;
function ensureWebp(): Promise<void> {
  if (!webpReady) {
    webpReady = webpInit({
      locateFile: (path: string) =>
        path.includes("simd") ? webpWasmSimd : webpWasmFallback,
    }).then(() => undefined);
  }
  return webpReady;
}

async function encodeWebp(
  rgba: Uint8Array,
  width: number,
  height: number,
  quality: number,
  lossless: boolean,
): Promise<Uint8Array> {
  await ensureWebp();
  // Dedicated ArrayBuffer copy: ImageData requires an ArrayBuffer-backed view.
  const data = new Uint8ClampedArray(rgba);
  const image = new ImageData(data, width, height);
  const options = lossless
    ? { lossless: 1 as const, exact: 1 as const, quality: 100 }
    : { lossless: 0 as const, exact: 0 as const, quality };
  const buffer = await webpEncode(image, options);
  return new Uint8Array(buffer);
}

// ---- handlers ---------------------------------------------------------------

async function handleLoad(id: number, bytes: Uint8Array): Promise<void> {
  const info = conv.loadImage(bytes);
  currentInfo = {
    width: info.width,
    height: info.height,
    frameCount: info.frameCount,
    animated: info.animated,
    format: info.format,
    delaysMs: info.delaysMs,
    loopCode: info.loopCode,
  };
  if (lastResult) {
    lastResult.free();
    lastResult = null;
  }
  post({ t: "loaded", id, info: currentInfo });
}

function handleConvert(id: number, frame: number, dto: ParamsDTO): void {
  const p = buildParams(dto);
  try {
    const t0 = performance.now();
    const res = conv.convertFrame(frame, p);
    const ms = performance.now() - t0;
    const dtoOut = toDTO(res);
    // Snapshot the presentation blend inputs for the renderer at THIS
    // conversion's asciiStrength (keeps CPU raster and GL pass consistent).
    const raster = conv.rasterBlended(res, frame, dto.asciiStrength);
    const source = conv.previewSource(res, frame);
    // Keep the wasm result alive for character exports of the shown frame.
    if (lastResult) lastResult.free();
    lastResult = res;
    post(
      {
        t: "result",
        id,
        frame,
        result: dtoOut,
        raster,
        source,
        ms,
        asciiStrength: dto.asciiStrength,
      },
      [
        dtoOut.glyphIndices.buffer as ArrayBuffer,
        dtoOut.cellForegroundRGBA.buffer as ArrayBuffer,
        dtoOut.cellBackgroundRGBA.buffer as ArrayBuffer,
        dtoOut.atlasRGBA.buffer as ArrayBuffer,
        raster.buffer as ArrayBuffer,
        source.buffer as ArrayBuffer,
      ],
    );
  } finally {
    p.free();
  }
}

async function handleGifExport(
  id: number,
  dto: ParamsDTO,
  still: boolean,
): Promise<void> {
  if (!currentInfo) throw new Error("no image loaded");
  const total = still ? 1 : currentInfo.frameCount;
  const p = buildParams(dto);
  let writer: GifWriter | null = null;
  try {
    for (let i = 0; i < total; i++) {
      if (cancelIds.has(id)) {
        cancelIds.delete(id);
        post({ t: "cancelled", id });
        return;
      }
      const res = conv.convertFrame(i, p);
      if (!writer) {
        const loopCode = still ? -1 : currentInfo.loopCode;
        writer = new GifWriter(res.outputWidth, res.outputHeight, loopCode);
      }
      const rgb = conv.rasterBlended(res, i, dto.asciiStrength);
      const delay = currentInfo.delaysMs[i] ?? 100;
      writer.addFrame(rgb, delay);
      res.free();
      post({ t: "progress", id, done: i + 1, total });
      // Cooperative cancellation point: flush queued messages (cancel) first.
      await yieldToEventLoop();
    }
    if (cancelIds.has(id)) {
      cancelIds.delete(id);
      post({ t: "cancelled", id });
      return;
    }
    if (!writer) throw new Error("GIF writer was not created");
    const bytes = writer.finish();
    writer = null;
    post(
      {
        t: "exported",
        id,
        kind: "gif",
        name: still ? "unscii-still.gif" : "unscii-animation.gif",
        mime: "image/gif",
        bytes,
      },
      [bytes.buffer as ArrayBuffer],
    );
  } finally {
    p.free();
    if (writer) {
      try {
        writer.free();
      } catch {
        /* consumed already */
      }
    }
  }
}

function requireResult(): ConversionResult {
  if (!lastResult) throw new Error("nothing converted yet");
  return lastResult;
}

function exportRaster(
  id: number,
  kind: "png" | "webp",
  rgba: Uint8Array,
  width: number,
  height: number,
  quality: number | undefined,
  lossless: boolean | undefined,
): void {
  if (kind === "png") {
    const bytes = conv.exportPng(rgba, width, height);
    post(
      { t: "exported", id, kind, name: "unscii.png", mime: "image/png", bytes },
      [bytes.buffer as ArrayBuffer],
    );
    return;
  }
  void encodeWebp(rgba, width, height, quality ?? 90, lossless ?? false)
    .then((bytes) =>
      post(
        {
          t: "exported",
          id,
          kind,
          name: lossless ? "unscii-lossless.webp" : "unscii-lossy.webp",
          mime: "image/webp",
          bytes,
        },
        [bytes.buffer as ArrayBuffer],
      ),
    )
    .catch((e) => post({ t: "error", id, message: errText(e) }));
}

function exportTextOrAnsi(id: number, kind: "text" | "ansi"): void {
  try {
    const res = requireResult();
    const text = kind === "text" ? conv.exportText(res) : conv.exportAnsi(res);
    const bytes = new TextEncoder().encode(text);
    post(
      {
        t: "exported",
        id,
        kind,
        name: kind === "text" ? "unscii.txt" : "unscii.ans",
        mime: "text/plain;charset=utf-8",
        bytes,
      },
      [bytes.buffer as ArrayBuffer],
    );
  } catch (e) {
    post({ t: "error", id, message: errText(e) });
  }
}

function exportHtml(id: number, woff: Uint8Array, colored: boolean): void {
  try {
    const res = requireResult();
    const html = conv.exportHtml(res, woff, colored);
    const bytes = new TextEncoder().encode(html);
    post(
      {
        t: "exported",
        id,
        kind: "html",
        name: "unscii.html",
        mime: "text/html;charset=utf-8",
        bytes,
      },
      [bytes.buffer as ArrayBuffer],
    );
  } catch (e) {
    post({ t: "error", id, message: errText(e) });
  }
}

// ---- message router ---------------------------------------------------------

ctx.onmessage = (event: MessageEvent<ToWorker>): void => {
  const msg = event.data;
  try {
    switch (msg.t) {
      case "load":
        void handleLoad(msg.id, msg.bytes).catch((e) =>
          post({ t: "error", id: msg.id, message: errText(e) }),
        );
        break;
      case "convert":
        handleConvert(msg.id, msg.frame, msg.params);
        break;
      case "cancel":
        cancelIds.add(msg.id);
        break;
      case "gifExport":
        void handleGifExport(msg.id, msg.params, msg.still).catch((e) => {
          cancelIds.delete(msg.id);
          post({ t: "error", id: msg.id, message: errText(e) });
        });
        break;
      case "exportText":
        exportTextOrAnsi(msg.id, msg.kind);
        break;
      case "exportHtml":
        exportHtml(msg.id, msg.woff, msg.colored);
        break;
      case "exportRaster":
        exportRaster(
          msg.id,
          msg.kind,
          msg.rgba,
          msg.width,
          msg.height,
          msg.quality,
          msg.lossless,
        );
        break;
      default: {
        const never: never = msg;
        throw new Error(`unknown message ${JSON.stringify(never)}`);
      }
    }
  } catch (e) {
    const id = "id" in msg ? (msg as { id: number }).id : 0;
    post({ t: "error", id, message: errText(e) });
  }
};
