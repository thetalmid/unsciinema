/**
 * Single WASM initialization point for the MAIN execution context.
 *
 * The worker has its own context and initializes independently inside
 * worker.ts (one init per context, as required). All entry points must go
 * through `wasmReady()` before touching wasm-exported symbols; errors from
 * initialization propagate to callers instead of being swallowed.
 */
import initWasm, {
  fontVariantsJson,
  palettePresetsJson,
} from "./generated/wasm/unsciinema_core.js";

export interface FontOption {
  /** Canonical variant name, e.g. "unscii-8-tall". */
  name: string;
  /** WOFF filename inside /fonts (public assets). */
  woff: string;
}

export interface PaletteOption {
  name: string;
  /** 0xRRGGBBAA integers. */
  colors: number[];
}

export interface WasmTables {
  fonts: FontOption[];
  palettes: PaletteOption[];
}

let readyPromise: Promise<WasmTables> | null = null;

/** Idempotent: the first call starts init, later calls await the same run. */
export function wasmReady(): Promise<WasmTables> {
  if (!readyPromise) {
    readyPromise = (async () => {
      await initWasm({
        module_or_path: new URL(
          "./generated/wasm/unsciinema_core_bg.wasm",
          import.meta.url,
        ),
      });
      const fonts = JSON.parse(fontVariantsJson()) as FontOption[];
      const palettes = JSON.parse(palettePresetsJson()) as PaletteOption[];
      if (fonts.length !== 7) throw new Error("expected 7 font variants");
      if (palettes.length !== 31) throw new Error("expected 31 palettes");
      return { fonts, palettes };
    })();
  }
  return readyPromise;
}

/** Base-safe URL for a variant WOFF shipped in public/fonts. */
export function woffUrl(font: FontOption): string {
  return `${import.meta.env.BASE_URL}fonts/${font.woff}`;
}

/** Fetch (same-origin) and cache WOFF bytes for HTML export. */
const woffCache = new Map<string, Uint8Array>();
export async function fetchWoff(font: FontOption): Promise<Uint8Array> {
  const hit = woffCache.get(font.name);
  if (hit) return hit;
  const res = await fetch(woffUrl(font));
  if (!res.ok) throw new Error(`font fetch failed: ${res.status} ${font.woff}`);
  const bytes = new Uint8Array(await res.arrayBuffer());
  woffCache.set(font.name, bytes);
  return bytes;
}
