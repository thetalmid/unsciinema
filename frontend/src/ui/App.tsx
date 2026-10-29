/**
 * App shell: worker lifecycle, generation/coalescing effects, file flow,
 * playback, keyboard cancel, and layout.
 *
 * Solid 2 notes:
 * - `createEffect(compute, apply)`: reactive reads ONLY in compute, side
 *   effects (worker posts, timers) in apply, cleanup returned from apply.
 * - listeners are registered in the owned `onSettled` scope with returned
 *   cleanup — never inside unowned ref callbacks.
 * - signal writes happen in handlers, worker callbacks, and apply steps.
 */
import { createEffect, onSettled, Show } from "solid-js";
import { Controls } from "./Controls";
import { ExportBar } from "./ExportBar";
import { PreviewPane } from "./PreviewPane";
import { StatusLine } from "./StatusLine";
import { WorkerClient } from "../worker/client";
import {
  convertMs,
  lastError,
  phase,
  playFrame,
  preview,
  publishPreview,
  setConvertMs,
  setGifProgress,
  setLastError,
  setPhase,
  setPlaying,
  setPlayFrame,
  setPreview,
  setSourceInfo,
  setSourceName,
  setStatusMessage,
  playing,
  sourceInfo,
} from "../state/app";
import { convertParams } from "../state/params";
import { bindExport, currentParamsDto, onExported } from "../exportController";
import { fetchWoff, type WasmTables } from "../wasmSetup";
import { getActiveManager } from "../render/active";

const MAX_INPUT_BYTES = 48 * 1024 * 1024; // mirrors the Rust input limit

export function App(props: { tables: WasmTables }) {
  const client = new WorkerClient({
    onLoaded(_id, info) {
      setSourceInfo(info);
      setPlayFrame(0);
      setPlaying(false);
      setPhase("ready");
      setLastError(null);
      setStatusMessage(
        `Loaded ${info.format} ${info.width}×${info.height}` +
          (info.animated
            ? ` · ${info.frameCount} frames · loop ${info.loopCode}`
            : ""),
      );
    },
    onResult(msg) {
      publishPreview({
        frame: msg.frame,
        result: msg.result,
        raster: msg.raster,
        source: msg.source,
        asciiStrength: msg.asciiStrength,
        convertMs: msg.ms,
      });
      setPhase((p) => (p === "exporting" || p === "loading" ? p : "ready"));
      setConvertMs(msg.ms);
      setLastError(null);
      setStatusMessage(
        `Converted frame ${msg.frame + 1} in ${msg.ms.toFixed(1)} ms · ` +
          `${msg.result.cols}×${msg.result.rows} cells · ` +
          `${msg.result.outputWidth}×${msg.result.outputHeight} px`,
      );
    },
    onProgress(_id, done, total) {
      setGifProgress({ done, total });
    },
    onExported(msg) {
      onExported(msg.bytes, msg.name, msg.mime);
    },
    onCancelled() {
      setPhase(preview() ? "ready" : "empty");
      setGifProgress(null);
      setStatusMessage("Cancelled");
    },
    onError(_id, message) {
      const loading = phase() === "loading";
      setGifProgress(null);
      setLastError(message);
      setStatusMessage(loading ? `Load failed: ${message}` : `Error: ${message}`);
      setPhase(
        loading
          ? sourceInfo()
            ? "ready"
            : "error"
          : preview()
            ? "ready"
            : "empty",
      );
    },
  });

  // Export controller binding: readback comes from the live renderer.
  bindExport({
    client,
    exportNative: () => {
      const mgr = getActiveManager();
      if (!mgr) throw new Error("renderer not ready");
      return mgr.exportNative();
    },
    fontWoff: async () => {
      const f = props.tables.fonts[convertParams().fontVariant];
      if (!f) throw new Error("unknown font variant");
      return fetchWoff(f);
    },
  });

  // ---- file flow ------------------------------------------------------------

  async function handleFile(file: File): Promise<void> {
    setLastError(null);
    if (file.size > MAX_INPUT_BYTES) {
      setPhase(sourceInfo() ? "ready" : "error");
      setStatusMessage(
        `Rejected ${file.name}: ${(file.size / 1024 / 1024).toFixed(1)} MiB exceeds the 48 MiB input limit`,
      );
      setLastError("Input limit: 48 MiB");
      return;
    }
    setPhase("loading");
    setStatusMessage(`Reading ${file.name}…`);
    try {
      const buf = await file.arrayBuffer();
      const bytes = new Uint8Array(buf);
      setSourceName(file.name);
      client.load(bytes); // client keeps its own copy for hard-restart reload
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setPhase(sourceInfo() ? "ready" : "error");
      setLastError(msg);
      setStatusMessage(`Failed to read file: ${msg}`);
    }
  }

  function clearSource(): void {
    client.clearRetained();
    client.hardRestart(); // drops worker-side decoded source too
    setSourceInfo(null);
    setSourceName("");
    setPlayFrame(0);
    setPlaying(false);
    setPreview(null);
    setConvertMs(null);
    setPhase("empty");
    setLastError(null);
    setStatusMessage("Source cleared");
  }

  // ---- owned lifecycle ------------------------------------------------------

  onSettled(() => {
    const onClearEvent = (): void => clearSource();
    window.addEventListener("unsciinema:clear", onClearEvent);

    const onKey = (e: KeyboardEvent): void => {
      if (e.key === "Escape") client.cancelActiveJob();
    };
    window.addEventListener("keydown", onKey);

    return () => {
      window.removeEventListener("unsciinema:clear", onClearEvent);
      window.removeEventListener("keydown", onKey);
      client.dispose();
    };
  });

  // ---- effects (compute -> apply) ------------------------------------------

  // Debounced conversion: one generation id per settled request; the worker
  // response is applied only if its id is still current (see WorkerClient).
  let convertTimer: ReturnType<typeof setTimeout> | undefined;
  createEffect(
    () => ({ info: sourceInfo(), p: convertParams(), f: playFrame() }),
    (snap) => {
      if (!snap.info) return;
      clearTimeout(convertTimer);
      const frame = Math.min(snap.f, Math.max(0, snap.info.frameCount - 1));
      const dto = currentParamsDto();
      convertTimer = setTimeout(() => {
        setPhase((p) =>
          p === "exporting" || p === "loading" || p === "error" ? p : "converting",
        );
        client.convert(frame, dto);
      }, 45);
      return () => clearTimeout(convertTimer);
    },
  );

  // Animation playback: schedule the next frame with the real GIF delay.
  createEffect(
    () => ({ play: playing(), info: sourceInfo(), f: playFrame() }),
    (snap) => {
      if (!snap.play || !snap.info || snap.info.frameCount < 2) return;
      const total = snap.info.frameCount;
      const delay = Math.max(20, snap.info.delaysMs[snap.f] ?? 100);
      const next = snap.f;
      const timer = setTimeout(() => {
        setPlayFrame((next + 1) % total);
      }, delay);
      return () => clearTimeout(timer);
    },
  );

  return (
    <div class="app">
      <header class="app-header">
        <h1>
          <span class="logo" aria-hidden="true">U</span> Unsciinema
          <span class="subtitle">Unscii image converter · 100% client-side</span>
        </h1>
        <a
          class="header-note"
          href="https://viznut.fi/unscii/"
          target="_blank"
          rel="noreferrer noopener"
        >
          Unscii by viznut
        </a>
      </header>

      <StatusLine />

      <div class="layout">
        <aside class="sidebar">
          <Controls tables={props.tables} />
        </aside>
        <main class="stage">
          <PreviewPane onFile={(f) => void handleFile(f)} onClear={clearSource} />
          <ExportBar />
        </main>
      </div>

      <footer class="app-footer">
        <span>
          No uploads, no analytics, no third-party requests — conversion runs
          in a local WebAssembly worker.
        </span>
        <Show when={lastError()}>
          {(err) => <span class="footer-err">Last error: {err()}</span>}
        </Show>
        <span class="footer-timing">
          <Show when={convertMs() !== null}>
            <>last convert {convertMs()!.toFixed(1)} ms</>
          </Show>
        </span>
        <span class="footer-phase">{phase()}</span>
      </footer>
    </div>
  );
}
