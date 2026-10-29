/**
 * Preview pane: canvas + renderer lifecycle (owned scope via `onSettled`),
 * file input + drag-and-drop, animation playback controls, renderer badge.
 *
 * Solid 2 rules honored: ref callbacks stay unowned (plain assignment only);
 * all listeners/timers/effects are registered in owned scopes with returned
 * cleanup; no onMount.
 */
import { createEffect, createSignal, onSettled, Show } from "solid-js";
import {
  convertMs,
  playFrame,
  playing,
  preview,
  renderMode,
  rendererNote,
  setPlayFrame,
  setPlaying,
  setRenderMode,
  setRendererNote,
  sourceInfo,
} from "../state/app";
import { crtParams } from "../state/params";
import { getActiveManager, setActiveManager } from "../render/active";
import { PreviewManager } from "../render/preview";

export function PreviewPane(props: { onFile: (file: File) => void; onClear: () => void }) {
  let canvasEl: HTMLCanvasElement | undefined;
  let fileInput: HTMLInputElement | undefined;
  const [dragging, setDragging] = createSignal(false);

  onSettled(() => {
    const canvas = canvasEl;
    if (!canvas) return;
    let manager: PreviewManager;
    try {
      manager = new PreviewManager(canvas, {
        onMode: (mode, note) => {
          setRenderMode(mode);
          setRendererNote(note);
        },
      });
    } catch {
      return;
    }
    setActiveManager(manager);

    // Initial draw with whatever snapshot already exists.
    const snap = preview();
    if (snap) {
      manager.setParams(crtParams());
      try {
        manager.draw({
          result: snap.result,
          source: snap.source,
          raster: snap.raster,
          asciiStrength: snap.asciiStrength,
        });
      } catch {
        /* surfaced through renderer notes */
      }
    }

    const onResize = (): void => {
      clearTimeout(resizeTimer);
      resizeTimer = setTimeout(() => manager.redrawOnly(), 80);
    };
    let resizeTimer: ReturnType<typeof setTimeout> | undefined;
    window.addEventListener("resize", onResize);

    return () => {
      clearTimeout(resizeTimer);
      window.removeEventListener("resize", onResize);
      if (getActiveManager() === manager) setActiveManager(null);
      manager.dispose();
    };
  });

  // Reactive draw: preview snapshots and CRT changes redraw without reconvert.
  createEffect(
    () => ({ snap: preview(), crt: crtParams() }),
    (v) => {
      const mgr = getActiveManager();
      if (!mgr || !v.snap) return;
      mgr.setParams(v.crt);
      try {
        mgr.draw({
          result: v.snap.result,
          source: v.snap.source,
          raster: v.snap.raster,
          asciiStrength: v.snap.asciiStrength,
        });
      } catch {
        /* renderer note covers failures */
      }
    },
  );

  function pickFiles(files: FileList | null): void {
    const f = files?.[0];
    if (f) props.onFile(f);
  }

  return (
    <div
      class={dragging() ? "preview-pane dragging" : "preview-pane"}
      onDragOver={(e) => {
        e.preventDefault();
        setDragging(true);
      }}
      onDragLeave={() => setDragging(false)}
      onDrop={(e) => {
        e.preventDefault();
        setDragging(false);
        pickFiles(e.dataTransfer?.files ?? null);
      }}
    >
      <div class="preview-toolbar">
        <span class={`badge badge-${renderMode()}`} data-testid="render-mode">
          {renderMode() === "webgl2"
            ? "WebGL2"
            : renderMode() === "canvas2d"
              ? "Canvas2D fallback"
              : "renderer…"}
        </span>
        <Show when={rendererNote()}>
          {(note) => <span class="renderer-note" role="status">{note()}</span>}
        </Show>
        <span class="timing">
          <Show when={convertMs() !== null} fallback={<span>—</span>}>
            <span data-testid="convert-ms">{convertMs()!.toFixed(1)} ms</span>
          </Show>
        </span>
        <span class="spacer" />
        <button
          type="button"
          class="btn btn-ghost"
          disabled={!sourceInfo()}
          onClick={() => fileInput?.click()}
        >
          Replace image…
        </button>
        <button
          type="button"
          class="btn btn-ghost"
          disabled={!preview()}
          onClick={props.onClear}
        >
          Clear
        </button>
      </div>

      <div class="canvas-wrap">
        <canvas
          ref={(el) => {
            canvasEl = el;
          }}
          class={preview() ? "preview-canvas" : "preview-canvas is-hidden"}
          data-testid="preview-canvas"
          aria-label="ASCII preview"
        />
        <Show when={!preview()}>
          <div class="dropzone" data-testid="dropzone">
            <p>
              <strong>Drop an image</strong> (PNG · JPEG · GIF · WebP)
            </p>
            <p class="field-hint">Decoded locally in Rust/WASM — nothing is uploaded.</p>
            <button type="button" class="btn" onClick={() => fileInput?.click()}>
              Choose file
            </button>
          </div>
        </Show>
        <input
          ref={(el) => {
            fileInput = el;
          }}
          type="file"
          accept="image/*"
          class="visually-hidden"
          aria-label="Choose image file"
          onChange={(e) => {
            pickFiles(e.currentTarget.files);
            e.currentTarget.value = "";
          }}
        />
      </div>

      <Show when={sourceInfo()?.animated}>
        <div class="anim-bar" role="group" aria-label="Animation playback">
          <button
            type="button"
            class="btn"
            onClick={() => setPlaying(!playing())}
            aria-pressed={playing() ? "true" : "false"}
          >
            {playing() ? "Pause" : "Play"}
          </button>
          <label for="anim-frame" class="inline-label">
            Frame
          </label>
          <input
            id="anim-frame"
            type="range"
            min={0}
            max={Math.max(0, (sourceInfo()?.frameCount ?? 1) - 1)}
            step={1}
            value={playFrame()}
            onInput={(e) => {
              setPlaying(false);
              setPlayFrame(Number(e.currentTarget.value));
            }}
          />
          <span class="field-value">
            {playFrame() + 1} / {sourceInfo()?.frameCount ?? 1}
            {" · "}
            {sourceInfo()?.delaysMs[playFrame()] ?? "?"} ms
          </span>
        </div>
      </Show>
    </div>
  );
}
