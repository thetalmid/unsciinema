/**
 * Application-level signals (Solid 2).
 *
 * Lifecycle rules honored here:
 * - signal writes live in event handlers / worker callbacks / effect apply
 *   steps, never in the component body;
 * - component bodies read signals only as initial snapshots (no top-level
 *   reactive reads);
 * - worker callbacks are plain events — safe places to write signals.
 */
import { createSignal } from "solid-js";
import type { ResultDTO, SourceInfoDTO } from "../worker/protocol";

export type Phase =
  | "empty"
  | "loading"
  | "ready"
  | "converting"
  | "exporting"
  | "error";

export type RenderMode = "uninitialized" | "webgl2" | "canvas2d";

export interface PreviewSnapshot {
  frame: number;
  result: ResultDTO;
  /** Blended CPU reference raster (outputWidth*outputHeight*4). */
  raster: Uint8Array;
  /** Source at output size (GL blend/texture input). */
  source: Uint8Array;
  asciiStrength: number;
  convertMs: number;
  /** Bumped whenever a new snapshot arrives (renderer redraw trigger). */
  version: number;
}

const [phase, setPhase] = createSignal<Phase>("empty");
const [statusMessage, setStatusMessage] = createSignal<string>("");
const [lastError, setLastError] = createSignal<string | null>(null);

const [sourceInfo, setSourceInfo] = createSignal<SourceInfoDTO | null>(null);
const [sourceName, setSourceName] = createSignal<string>("");
/** Raw bytes kept on the main thread so a hard-cancel restart can reload. */
const [sourceBytes, setSourceBytes] = createSignal<Uint8Array | null>(null);

const [preview, setPreview] = createSignal<PreviewSnapshot | null>(null);
const [convertMs, setConvertMs] = createSignal<number | null>(null);

const [playFrame, setPlayFrame] = createSignal(0);
const [playing, setPlaying] = createSignal(false);

const [renderMode, setRenderMode] = createSignal<RenderMode>("uninitialized");
const [rendererNote, setRendererNote] = createSignal<string | null>(null);

const [gifProgress, setGifProgress] = createSignal<{
  done: number;
  total: number;
} | null>(null);

export {
  phase,
  setPhase,
  statusMessage,
  setStatusMessage,
  lastError,
  setLastError,
  sourceInfo,
  setSourceInfo,
  sourceName,
  setSourceName,
  sourceBytes,
  setSourceBytes,
  preview,
  setPreview,
  convertMs,
  setConvertMs,
  playFrame,
  setPlayFrame,
  playing,
  setPlaying,
  renderMode,
  setRenderMode,
  rendererNote,
  setRendererNote,
  gifProgress,
  setGifProgress,
};

let previewVersion = 0;
/** Monotonic count of applied conversions (test/diagnostic hook). */
const [resultSeq, setResultSeq] = createSignal(0);
export { resultSeq, setResultSeq };

/** Helper to push a fresh preview snapshot (worker callback). */
export function publishPreview(snap: Omit<PreviewSnapshot, "version">): void {
  previewVersion += 1;
  setPreview({ ...snap, version: previewVersion });
  setConvertMs(snap.convertMs);
  setResultSeq((n) => n + 1);
}
