/**
 * The complete control panel (mission §7 inventory):
 * seven fonts, cell size, target width, contrast, gamma, dither
 * algorithm/strength/threshold, ASCII/original blend, color mode,
 * presets/custom palette, alpha/background, and CRT scanlines, curvature,
 * bloom, vignette — defaults valid and neutral where appropriate.
 */
import { createSignal, Show } from "solid-js";
import type { WasmTables } from "../wasmSetup";
import {
  clamp,
  convertParams,
  crtParams,
  resetAllParams,
  setConvert,
  setCrt,
} from "../state/params";
import {
  phase,
  renderMode,
  setPhase,
  sourceInfo,
  sourceName,
  setStatusMessage,
} from "../state/app";
import { Group, NumberField, Select, Slider } from "./Field";

const DITHER_OPTIONS = [
  { value: 0, label: "None" },
  { value: 1, label: "Floyd–Steinberg" },
  { value: 2, label: "Atkinson" },
  { value: 3, label: "Bayer 4×4" },
  { value: 4, label: "Bayer 8×8" },
  { value: 5, label: "Random Noise" },
  { value: 6, label: "Sierra" },
  { value: 7, label: "Jarvis–Judice–Ninke" },
  { value: 8, label: "Stucki" },
  { value: 9, label: "Burkes" },
  { value: 10, label: "Blue Noise (void-and-cluster)" },
] as const;

const COLOR_OPTIONS = [
  { value: 0, label: "Monochrome" },
  { value: 1, label: "Original color" },
  { value: 2, label: "Palette" },
] as const;

const ALPHA_OPTIONS = [
  { value: 0, label: "Ignore source alpha" },
  { value: 1, label: "Preserve transparency" },
  { value: 2, label: "Composite over background" },
] as const;

/** Parse "#RRGGBB[#AA]" tokens (comma/whitespace/newline separated). */
function parseHexPalette(text: string): { colors: number[]; error: string | null } {
  const tokens = text.split(/[\s,;]+/).filter(Boolean);
  if (tokens.length === 0) return { colors: [], error: "Palette is empty" };
  if (tokens.length > 64) {
    return { colors: [], error: "Maximum 64 colors (got " + tokens.length + ")" };
  }
  const colors: number[] = [];
  for (const tok of tokens) {
    const m = /^(?:#?)([0-9a-fA-F]{6})([0-9a-fA-F]{2})?$/.exec(tok);
    if (!m) return { colors: [], error: `Invalid color "${tok}" (use #RRGGBB or #RRGGBBAA)` };
    const rgb = parseInt(m[1], 16);
    const a = m[2] ? parseInt(m[2], 16) : 0xff;
    colors.push((((rgb << 8) | a) >>> 0));
  }
  if (colors.length < 1) return { colors: [], error: "Palette needs at least 1 color" };
  return { colors, error: null };
}

export function Controls(props: { tables: WasmTables }) {
  const [customText, setCustomText] = createSignal(
    "#101418 #2b3440 #4c5a68 #7b8794 #b7c2cd #ffffff",
  );
  const [customError, setCustomError] = createSignal<string | null>(null);

  const fontOptions = () =>
    props.tables.fonts.map((f, i) => ({ value: i, label: f.name }));
  const paletteOptions = () => {
    const opts = props.tables.palettes.map((p, i) => ({
      value: i,
      label: `${p.name} (${p.colors.length})`,
    }));
    opts.push({ value: -1, label: "Custom palette…" });
    return opts;
  };

  const hasSource = () => phase() !== "empty";
  // CRT effects are GPU-only: disabled AND identified in Canvas2D fallback.
  const crtDisabled = () => renderMode() === "canvas2d";

  function applyCustomPalette(text: string): void {
    setCustomText(text);
    const { colors, error } = parseHexPalette(text);
    setCustomError(error);
    if (!error) {
      setConvert("customPalette", new Uint32Array(colors));
      setConvert("paletteIndex", -1);
    }
  }

  return (
    <div class="controls">
      <Group title="Source">
        <Show
          when={sourceInfo()}
          fallback={<p class="field-hint">No image — drop a PNG, JPEG, GIF, or WebP anywhere on the preview.</p>}
        >
          {(info) => (
            <div class="source-info">
              <div class="source-name" title={sourceName()}>{sourceName()}</div>
              <div class="source-meta">
                {info().width}×{info().height} · {info().format}
                {info().animated ? ` · ${info().frameCount} frames` : ""}
              </div>
              <div class="row">
                <button
                  type="button"
                  class="btn btn-ghost"
                  onClick={() => {
                    window.dispatchEvent(new CustomEvent("unsciinema:clear"));
                  }}
                >
                  Clear source
                </button>
                <button
                  type="button"
                  class="btn btn-ghost"
                  onClick={() => {
                    resetAllParams();
                    setStatusMessage("Settings reset to defaults");
                    if (phase() === "error") setPhase(sourceInfo() ? "ready" : "empty");
                  }}
                >
                  Reset settings
                </button>
              </div>
            </div>
          )}
        </Show>
      </Group>

      <Group title="Font" note="unscii-16 glyphs are 8×16; tall/thin change cell aspect.">
        <Select
          id="ctl-font"
          label="Variant"
          value={convertParams().fontVariant}
          options={fontOptions()}
          onChange={(v) => setConvert("fontVariant", v)}
        />
      </Group>

      <Group title="Sampling">
        <Slider
          id="ctl-cell"
          label="Cell size (source px per cell)"
          value={convertParams().cellSize}
          min={1}
          max={64}
          step={0.5}
          display={(v) => (Number.isInteger(v) ? String(v) : v.toFixed(1))}
          onInput={(v) => setConvert("cellSize", clamp(v, 1, 64))}
          hint="Fractional values are sampled deterministically, partial edge cells included."
        />
        <NumberField
          id="ctl-target"
          label="Target width (px, 0 = auto)"
          value={convertParams().targetWidth}
          min={0}
          max={8192}
          onInput={(v) => setConvert("targetWidth", Math.round(clamp(v, 0, 8192)))}
          hint="0 keeps the original width; positive = exact width, aspect preserved."
        />
      </Group>

      <Group
        title="Image"
        note="Contrast pivots at 128 (linear); gamma is pow(c, 1/γ) in encoded sRGB space. Both neutral at 1.0."
      >
        <Slider
          id="ctl-contrast"
          label="Contrast"
          value={convertParams().contrast}
          min={0.1}
          max={3}
          step={0.01}
          display={(v) => v.toFixed(2)}
          onInput={(v) => setConvert("contrast", clamp(v, 0.1, 3))}
        />
        <Slider
          id="ctl-gamma"
          label="Gamma"
          value={convertParams().gamma}
          min={0.5}
          max={2.5}
          step={0.01}
          display={(v) => v.toFixed(2)}
          onInput={(v) => setConvert("gamma", clamp(v, 0.5, 2.5))}
        />
      </Group>

      <Group
        title="Dither"
        note="Applied to cell luminance against the 16-level glyph ramp. Strength 0 ≡ None. ID 5 is labeled Random Noise (xorshift); Blue Noise is a separate option."
      >
        <Select
          id="ctl-dither"
          label="Algorithm"
          value={convertParams().ditherAlgorithm}
          options={DITHER_OPTIONS}
          onChange={(v) => setConvert("ditherAlgorithm", v)}
        />
        <Slider
          id="ctl-strength"
          label="Dither strength"
          value={convertParams().ditherStrength}
          min={0}
          max={1}
          step={0.01}
          display={(v) => v.toFixed(2)}
          onInput={(v) => setConvert("ditherStrength", clamp(v, 0, 1))}
        />
        <Slider
          id="ctl-threshold"
          label="Threshold"
          value={convertParams().threshold}
          min={0}
          max={1}
          step={0.01}
          display={(v) => v.toFixed(2)}
          onInput={(v) => setConvert("threshold", clamp(v, 0, 1))}
        />
      </Group>

      <Group
        title="ASCII / original blend"
        note="Presentation strength, separate from dithering: 0 = original image, 1 = full ASCII (default). Raster exports follow it; character-only exports cannot."
      >
        <Slider
          id="ctl-blend"
          label="ASCII strength"
          value={convertParams().asciiStrength}
          min={0}
          max={1}
          step={0.01}
          display={(v) => (v === 0 ? "0 (original)" : v === 1 ? "1 (ASCII)" : v.toFixed(2))}
          onInput={(v) => setConvert("asciiStrength", clamp(v, 0, 1))}
        />
      </Group>

      <Group title="Color">
        <Select
          id="ctl-color"
          label="Color mode"
          value={convertParams().colorMode}
          options={COLOR_OPTIONS}
          onChange={(v) => setConvert("colorMode", v)}
        />
        <Select
          id="ctl-palette"
          label="Palette preset"
          value={convertParams().paletteIndex}
          options={paletteOptions()}
          disabled={convertParams().colorMode !== 2}
          onChange={(v) => {
            if (v >= 0) {
              setConvert("paletteIndex", v);
              setConvert("customPalette", null);
              setCustomError(null);
            } else {
              setConvert("paletteIndex", -1);
            }
          }}
          hint="31 presets from the Rust table (≥8-color palettes fully supported)."
        />
        <Show when={convertParams().colorMode === 2 && convertParams().paletteIndex === -1}>
          <div class="field">
            <label for="ctl-custom">Custom palette (#RRGGBB or #RRGGBBAA)</label>
            <textarea
              id="ctl-custom"
              rows={3}
              spellcheck={false}
              value={customText()}
              onInput={(e) => applyCustomPalette(e.currentTarget.value)}
              aria-invalid={customError() !== null ? "true" : "false"}
              aria-describedby="ctl-custom-err"
            />
            <p id="ctl-custom-err" class="field-error" role={customError() ? "alert" : undefined}>
              {customError() ?? ""}
            </p>
          </div>
        </Show>
        <Select
          id="ctl-alpha"
          label="Alpha mode"
          value={convertParams().alphaMode}
          options={ALPHA_OPTIONS}
          onChange={(v) => setConvert("alphaMode", v)}
          hint="Ignore / Preserve / Composite over background — end-to-end incl. exports."
        />
        <div class="field">
          <label for="ctl-bg">Background color</label>
          <div class="row">
            <input
              id="ctl-bg"
              type="color"
              value={`#${(convertParams().background >>> 8 & 0xffffff).toString(16).padStart(6, "0")}`}
              onInput={(e) => {
                const rgb = parseInt(e.currentTarget.value.slice(1), 16);
                const a = convertParams().background & 0xff;
                setConvert("background", (((rgb << 8) | a) >>> 0));
              }}
            />
            <label for="ctl-bg-alpha" class="inline-label">Alpha</label>
            <input
              id="ctl-bg-alpha"
              type="range"
              min={0}
              max={255}
              step={1}
              value={convertParams().background & 0xff}
              onInput={(e) => {
                const a = Number(e.currentTarget.value);
                setConvert("background", (((convertParams().background >>> 8) << 8 | a) >>> 0));
              }}
            />
            <output for="ctl-bg-alpha" class="field-value">
              {convertParams().background & 0xff}
            </output>
          </div>
          <p class="field-hint">0xRRGGBBAA shifts are explicit; alpha applies per alpha mode.</p>
        </div>
      </Group>

      <Group
        title="CRT effects (WebGL2)"
        note={crtDisabled() ? "Unavailable in CPU fallback mode." : "Zero = off = pixel-exact neutral. Effects are baked into PNG/WebP exports via the native-resolution post framebuffer."}
      >
        <Slider
          id="crt-scan"
          label="Scanlines"
          value={crtParams().scanlines}
          min={0}
          max={1}
          step={0.01}
          display={(v) => (v === 0 ? "off" : v.toFixed(2))}
          disabled={crtDisabled()}
          onInput={(v) => setCrt("scanlines", clamp(v, 0, 1))}
        />
        <Slider
          id="crt-curve"
          label="Curvature"
          value={crtParams().curvature}
          min={0}
          max={1}
          step={0.01}
          display={(v) => (v === 0 ? "off" : v.toFixed(2))}
          disabled={crtDisabled()}
          onInput={(v) => setCrt("curvature", clamp(v, 0, 1))}
        />
        <Slider
          id="crt-bloom"
          label="Bloom"
          value={crtParams().bloom}
          min={0}
          max={1}
          step={0.01}
          display={(v) => (v === 0 ? "off" : v.toFixed(2))}
          disabled={crtDisabled()}
          onInput={(v) => setCrt("bloom", clamp(v, 0, 1))}
        />
        <Slider
          id="crt-vig"
          label="Vignette"
          value={crtParams().vignette}
          min={0}
          max={1}
          step={0.01}
          display={(v) => (v === 0 ? "off" : v.toFixed(2))}
          disabled={crtDisabled()}
          onInput={(v) => setCrt("vignette", clamp(v, 0, 1))}
        />
      </Group>

      <Show when={hasSource()}>
        <p class="field-hint controls-foot">
          Changes recompute in the worker; the previous preview stays visible.
        </p>
      </Show>
    </div>
  );
}
