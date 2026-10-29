/**
 * Export bar: PNG, lossy/lossless WebP, text, HTML, ANSI, GIF.
 * Character-only exports are labeled with their inherent limitation
 * (no colors/CRT/raster blend — they are the glyph grid itself).
 */
import { Show } from "solid-js";
import {
  exportGif,
  cancelGif,
  exportHtml,
  exportRaster,
  exportTextOrAnsi,
  setWebpLossless,
  setWebpQuality,
  webpLossless,
  webpQuality,
} from "../exportController";
import { gifProgress, phase, preview, sourceInfo } from "../state/app";

export function ExportBar() {
  const disabled = () => !preview() || phase() === "converting" || phase() === "loading";
  const gifBusy = () => gifProgress() !== null;

  return (
    <div class="export-bar" role="group" aria-label="Exports">
      <div class="row wrap">
        <button
          type="button"
          class="btn"
          data-testid="export-png"
          disabled={disabled()}
          onClick={() => exportRaster("png")}
          title="Native-resolution PNG including CRT effects and the ASCII blend"
        >
          PNG
        </button>
        <span class="btn-group">
          <button
            type="button"
            class="btn"
            data-testid="export-webp-lossy"
            disabled={disabled() || webpLossless()}
            onClick={() => exportRaster("webp")}
          >
            WebP lossy
          </button>
          <label for="webp-q" class="inline-label">
            q
          </label>
          <input
            id="webp-q"
            class="webp-q"
            type="number"
            min={1}
            max={100}
            step={1}
            value={webpQuality()}
            disabled={webpLossless()}
            onInput={(e) => setWebpQuality(Math.min(100, Math.max(1, Number(e.currentTarget.value) || 90)))}
            aria-label="WebP quality"
          />
        </span>
        <span class="btn-group">
          <label class="inline-label" for="webp-lossless">
            <input
              id="webp-lossless"
              type="checkbox"
              checked={webpLossless()}
              onChange={(e) => setWebpLossless(e.currentTarget.checked)}
            />
            lossless
          </label>
          <button
            type="button"
            class="btn"
            data-testid="export-webp-lossless"
            disabled={disabled() || !webpLossless()}
            onClick={() => exportRaster("webp")}
            title="libwebp lossless with exact=1 (RGB under full transparency preserved)"
          >
            WebP lossless
          </button>
        </span>
        <button
          type="button"
          class="btn"
          data-testid="export-text"
          disabled={disabled()}
          onClick={() => exportTextOrAnsi("text")}
        >
          Text
        </button>
        <button
          type="button"
          class="btn"
          data-testid="export-html"
          disabled={disabled()}
          onClick={() => void exportHtml()}
        >
          HTML
        </button>
        <button
          type="button"
          class="btn"
          data-testid="export-ansi"
          disabled={disabled()}
          onClick={() => exportTextOrAnsi("ansi")}
        >
          ANSI
        </button>
        <button
          type="button"
          class="btn"
          data-testid="export-gif"
          disabled={disabled() || gifBusy()}
          onClick={exportGif}
        >
          {sourceInfo()?.animated ? "GIF (animation)" : "GIF (still)"}
        </button>
        <Show when={gifBusy()}>
          <button type="button" class="btn btn-danger" data-testid="cancel-gif" onClick={cancelGif}>
            Cancel GIF
          </button>
        </Show>
      </div>

      <Show when={gifProgress()}>
        {(p) => (
          <div class="gif-progress" role="status" data-testid="gif-progress">
            <progress value={p().done} max={p().total} />
            <span>
              frame {p().done}/{p().total}
            </span>
          </div>
        )}
      </Show>

      <p class="field-hint export-note">
        PNG/WebP = what you see (blend + CRT, native resolution, GPU readback
        or CPU raster). Text/ANSI/HTML = the character grid itself — colors
        yes (HTML/ANSI), CRT/original blend no. GIF = blended ASCII frames
        with real delays/loop, CRT not baked in.
      </p>
    </div>
  );
}
