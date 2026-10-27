---
id: arch-unscii-converter
type: architecture-design
title: Unscii Image Converter Architecture
status: active
parent: goal-unscii-converter
---

# Unscii Image Converter Architecture

## Overview

This document defines the module breakdown, data flow, and key interfaces for the Unscii Image Converter — a Rust/WASM + SolidJS v2 web app for converting images to ASCII art using the unscii font with real-time artistic controls.

**Target Stack:** Rust → WASM (wasm-pack) + SolidJS v2 (Vite) + WebGL2/WGSL + Static Hosting

---

## Module Breakdown

```
unscii-converter/
├── core/                          # Rust crate → WASM (pkg/)
│   ├── src/
│   │   ├── lib.rs                 # WASM exports (public API)
│   │   ├── font/
│   │   │   ├── mod.rs
│   │   │   ├── parser.rs          # .hex format parsing
│   │   │   ├── atlas.rs           # Font atlas generation (128×8 texture)
│   │   │   └── variants.rs        # Variant definitions (unscii-8, alt, fantasy, etc.)
│   │   ├── image/
│   │   │   ├── mod.rs
│   │   │   ├── loader.rs          # PNG/JPEG/GIF/WebP decoding via image crate
│   │   │   ├── luminance.rs       # BT.709 conversion (RGBA → Luma)
│   │   │   └── preprocessing.rs   # Contrast, gamma, resize (Catmull-Rom)
│   │   ├── dither/
│   │   │   ├── mod.rs
│   │   │   ├── error_diffusion.rs # Floyd-Steinberg, Atkinson, Sierra, JJN, Stucki, Burkes
│   │   │   ├── ordered.rs         # Bayer 4×4, Bayer 8×8, Blue Noise (CPU fallback)
│   │   │   └── palette.rs         # 31 preset palettes + custom palette mapping
│   │   ├── convert/
│   │   │   ├── mod.rs
│   │   │   ├── mapper.rs          # Luma → glyph index (density mapping)
│   │   │   └── grid.rs            # ASCII grid structure (Vec<u16>, cols, rows)
│   │   └── export/
│   │       ├── mod.rs
│   │       ├── png.rs             # PNG encoder (png crate)
│   │       ├── webp.rs            # WebP encoder (zenwebp)
│   │       ├── text.rs            # Plain text export
│   │       ├── html.rs            # HTML export with embedded styles
│   │       └── ansi.rs            # ANSI 24-bit color export
│   ├── Cargo.toml
│   └── .rsw.json                  # rsw config for SolidJS integration
│
├── frontend/                      # SolidJS + Vite app
│   ├── src/
│   │   ├── index.tsx              # App mount
│   │   ├── App.tsx                # Root component
│   │   ├── components/
│   │   │   ├── Canvas.tsx         # WebGL canvas + render loop
│   │   │   ├── ControlPanel.tsx   # All interactive controls
│   │   │   ├── FileDrop.tsx       # Drag-and-drop + file input
│   │   │   ├── ExportModal.tsx    # Export options + download
│   │   │   └── StatusBar.tsx      # Info: dimensions, mode, perf
│   │   ├── hooks/
│   │   │   ├── useUnscii.ts       # WASM init, converter instance
│   │   │   ├── useWebGL.ts        # WebGL2 context, shader pipeline
│   │   │   ├── useImage.ts        # Image loading, FileReader
│   │   │   └── useExport.ts       # Blob download helpers
│   │   ├── shaders/
│   │   │   ├── ascii.wgsl         # ASCII compute shader (main)
│   │   │   ├── luma.wgsl          # BT.709 luminance pass
│   │   │   ├── dither.wgsl        # Ordered dithering (Bayer/Blue Noise)
│   │   │   └── postprocess.wgsl   # CRT effects (scanlines, curvature, bloom, vignette)
│   │   ├── stores/
│   │   │   ├── params.ts          # Reactive parameters (Solid signals)
│   │   │   └── state.ts           # App state (image, grid, loading)
│   │   └── styles/
│   │       ├── app.css
│   │       └── control-panel.css
│   ├── index.html
│   ├── vite.config.ts
│   ├── package.json
│   └── tsconfig.json
│
├── public/
│   └── fonts/
│       ├── unscii-8.hex           # Primary 8×8 font
│       ├── unscii-16.hex          # 8×16 font
│       ├── unscii-8-alt.hex       # Alt variant
│       ├── unscii-8-fantasy.hex   # Fantasy variant
│       ├── unscii-8-mcr.hex       # MCR variant
│       ├── unscii-8-tall.hex      # Tall variant
│       ├── unscii-8-thin.hex      # Thin variant
│       └── unscii-8.woff          # For HTML export @font-face
│
├── research/                      # Research reference (not in build)
│   └── ...
│
├── goal-and-requirements.md
├── architecture.md                # This file
├── AGENTS.md
└── README.md
```

---

## Data Flow

### Primary Pipeline (Per Frame)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           RUST / WASM (core)                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  User Image (File/Blob)                                                     │
│        │                                                                    │
│        ▼                                                                    │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐ │
│  │  loader.rs  │───▶│ luminance.rs│───▶│  dither/    │───▶│  mapper.rs  │ │
│  │  decode()   │    │  to_luma()  │    │  apply()    │    │  map_grid() │ │
│  └─────────────┘    └─────────────┘    └─────────────┘    └──────┬──────┘ │
│        │                │                │                       │        │
│        ▼                ▼                ▼                       ▼        │
│  DynamicImage      Vec<u8>           Vec<u8>               AsciiGrid   │
│  (RGBA)            (Luma, W×H)       (Dithered Luma)      (u16[], W,H)  │
│                                                                             │
└──────────────────────────────────┬────────────────────────────────────────┘
                                   │
                                   │ wasm-bindgen (Vec<u16> + metadata)
                                   ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                        JAVASCRIPT / SOLIDJS (frontend)                      │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  AsciiGrid (u16[], cols, rows)                                              │
│        │                                                                    │
│        ▼                                                                    │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐ │
│  │ useWebGL.ts │───▶│  ascii.wgsl │───▶│  Canvas     │───▶│  Display    │ │
│  │  upload()   │    │  compute()  │    │  readback   │    │  (request   │ │
│  └─────────────┘    └─────────────┘    └─────────────┘    │  animation) │ │
│        │                │                │                └─────────────┘ │
│        │                │                │                                  │
│        ▼                ▼                ▼                                  │
│  Font Atlas       Uniforms         RGBA16Float                           │
│  (128×8 texture)  (cell_size,    Texture (GPU)                           │
│   + glyph data)   strength, etc)                                          │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Export Pipeline (On Demand)

```
AsciiGrid + RGBA (from WASM) + Params
        │
        ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  Rust/WASM Export Functions                                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  export_png(rgba, w, h)          → Vec<u8> (PNG bytes)                     │
│  export_webp(rgba, w, h, q, ll)  → Vec<u8> (WebP bytes)                    │
│  export_text(grid, cols, rows)   → String (plain text)                     │
│  export_html(grid, cols, rows,   → String (HTML doc)                       │
│      fg, bg, font, lh, fs)                                                  │
│  export_ansi(grid, cols, rows,   → String (ANSI escape codes)              │
│      fg_rgb, bg_rgb)                                                         │
│                                                                             │
│  GIF: Frames exported from WASM → JS gifski-wasm.encode() → Blob            │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Key Interfaces

### WASM Exports (core/src/lib.rs)

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct UnsciiConverter {
    font_atlas: Vec<u8>,           // 128×8 RGBA atlas (GPU upload)
    font_width: u32,               // 8
    font_height: u32,              // 8
    glyph_count: u32,              // 16
    char_ramp: Vec<char>,          // Density-ordered chars
}

#[wasm_bindgen]
impl UnsciiConverter {
    /// Create converter from unscii .hex font data
    #[wasm_bindgen(constructor)]
    pub fn new(font_hex: &str, variant: FontVariant) -> Result<UnsciiConverter, JsValue>;

    /// Get font atlas as RGBA bytes for GPU upload (128×8 = 4096 bytes)
    #[wasm_bindgen]
    pub fn font_atlas(&self) -> Vec<u8>;

    /// Get font atlas dimensions
    #[wasm_bindgen]
    pub fn font_atlas_size(&self) -> Vec<u32>;  // [width, height] = [128, 8]

    /// Get character ramp (density-ordered)
    #[wasm_bindgen]
    pub fn char_ramp(&self) -> Vec<u16>;  // UTF-16 code units

    /// Process image: decode → luminance → dither → map to glyphs
    /// Returns: { grid: Vec<u16>, cols: u32, rows: u32, rgba: Vec<u8> }
    #[wasm_bindgen]
    pub fn convert(
        &self,
        image_bytes: &[u8],
        params: &ConvertParams,
    ) -> Result<ConvertResult, JsValue>;

    /// Export current result as PNG
    #[wasm_bindgen]
    pub fn export_png(&self, rgba: &[u8], width: u32, height: u32) -> Vec<u8>;

    /// Export as WebP (lossy or lossless)
    #[wasm_bindgen]
    pub fn export_webp(&self, rgba: &[u8], width: u32, height: u32, quality: f32, lossless: bool) -> Vec<u8>;

    /// Export as plain text
    #[wasm_bindgen]
    pub fn export_text(&self, grid: &[u16], cols: u32, rows: u32) -> String;

    /// Export as HTML document
    #[wasm_bindgen]
    pub fn export_html(
        &self,
        grid: &[u16],
        cols: u32,
        rows: u32,
        fg_color: &str,
        bg_color: &str,
        font_family: &str,
        line_height: f32,
        font_size: u32,
    ) -> String;

    /// Export as ANSI escape codes
    #[wasm_bindgen]
    pub fn export_ansi(
        &self,
        grid: &[u16],
        cols: u32,
        rows: u32,
        fg_rgb: u32,
        bg_rgb: u32,
    ) -> String;

    /// Get frames for GIF export (RGBA frames)
    #[wasm_bindgen]
    pub fn get_gif_frames(&self, grids: &[Vec<u16>], cols: u32, rows: u32, palette: &[u32]) -> Vec<Vec<u8>>;
}

#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct ConvertParams {
    // Dithering
    pub dither_algorithm: u32,      // 0=none, 1=FS, 2=Atkinson, 3=Bayer4, 4=Bayer8, 5=BlueNoise, 6=Sierra, 7=JJN, 8=Stucki, 9=Burkes
    pub dither_strength: f32,       // 0.0–1.0 blend factor
    pub threshold: f32,             // 0.0–1.0 global threshold

    // Font
    pub font_variant: u32,          // 0=unscii-8, 1=alt, 2=fantasy, 3=mcr, 4=tall, 5=thin, 6=unscii-16

    // Rendering
    pub cell_size: f32,             // 4.0–32.0 pixels per glyph
    pub color_mode: u32,            // 0=mono, 1=color, 2=palette
    pub palette: Vec<u32>,          // Packed RGBA (up to 6 colors)
    pub palette_size: u32,          // Number of colors in palette

    // Alpha
    pub alpha_mode: u32,            // 0=ignore, 1=transparent, 2=blend

    // Pre-processing
    pub contrast: f32,              // 0.1–3.0
    pub gamma: f32,                 // 0.5–2.5
    pub target_width: u32,          // 0 = auto (based on cell_size)

    // Post-processing (CRT)
    pub crt_scanlines: bool,
    pub crt_curvature: f32,         // 0.0–1.0
    pub crt_bloom: f32,             // 0.0–1.0
    pub crt_vignette: f32,          // 0.0–1.0
}

#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct ConvertResult {
    pub grid: Vec<u16>,             // Glyph indices per cell (row-major)
    pub cols: u32,                  // Grid width in cells
    pub rows: u32,                  // Grid height in cells
    pub rgba: Vec<u8>,              // Rendered RGBA (cols*8 × rows*8 × 4)
    pub width: u32,                 // Image width in pixels
    pub height: u32,                // Image height in pixels
}

#[wasm_bindgen]
pub enum FontVariant {
    Unscii8 = 0,
    Unscii8Alt = 1,
    Unscii8Fantasy = 2,
    Unscii8Mcr = 3,
    Unscii8Tall = 4,
    Unscii8Thin = 5,
    Unscii16 = 6,
}
```

### WebGL/WGSL Shader Interface

#### Uniform Buffer (ascii.wgsl)
```wgsl
struct AsciiParams {
    cell_size: f32,      // Pixel size of each glyph cell
    strength: f32,       // 0.0 = original, 1.0 = full ASCII
    color_mode: u32,     // 0=mono, 1=color, 2=palette
    _padding: f32,
}

struct PaletteData {
    colors: array<vec4<f32>, 6>,  // Up to 6 colors (RGBA 0-1)
    count: u32,
    _padding: vec2<u32>,
}

@group(0) @binding(0) var source_tex: texture_2d<f32>;
@group(0) @binding(1) var luma_tex: texture_2d<f32>;
@group(0) @binding(2) var output_tex: texture_storage_2d<rgba16float, write>;
@group(1) @binding(0) var<uniform> params: AsciiParams;
@group(1) @binding(1) var<uniform> palette: PaletteData;
@group(2) @binding(0) var char_atlas: texture_2d<f32>;
@group(2) @binding(1) var atlas_sampler: sampler;
```

#### Compute Shader Dispatch
```javascript
// Dispatch: one workgroup per 16×16 pixel block
const workgroupSize = 16;
const dispatchX = Math.ceil(canvasWidth / workgroupSize);
const dispatchY = Math.ceil(canvasHeight / workgroupSize);
computePass.dispatchWorkgroups(dispatchX, dispatchY);
```

### SolidJS Reactive Parameters (frontend/src/stores/params.ts)

```typescript
import { createSignal, createStore, produce } from 'solid-js';

export interface ConvertParams {
  // Dithering
  ditherAlgorithm: DitherAlgorithm;
  ditherStrength: number;      // 0–1
  threshold: number;           // 0–1

  // Font
  fontVariant: FontVariant;

  // Rendering
  cellSize: number;            // 4–32
  colorMode: ColorMode;
  palette: number[];           // Packed RGBA (0xRRGGBBAA)
  paletteSize: number;         // 2–6

  // Alpha
  alphaMode: AlphaMode;

  // Pre-processing
  contrast: number;            // 0.1–3.0
  gamma: number;               // 0.5–2.5
  targetWidth: number;         // 0 = auto

  // CRT
  crtScanlines: boolean;
  crtCurvature: number;        // 0–1
  crtBloom: number;            // 0–1
  crtVignette: number;         // 0–1
}

// Reactive store with derived signals
export const [params, setParams] = createStore<ConvertParams>({
  ditherAlgorithm: 'none',
  ditherStrength: 1.0,
  threshold: 0.5,
  fontVariant: 'unscii-8',
  cellSize: 8,
  colorMode: 'mono',
  palette: [0x000000ff, 0xffffffff],  // Black, white
  paletteSize: 2,
  alphaMode: 'ignore',
  contrast: 1.0,
  gamma: 1.0,
  targetWidth: 0,
  crtScanlines: false,
  crtCurvature: 0.0,
  crtBloom: 0.0,
  crtVignette: 0.0,
});

// Derived: auto-computed grid dimensions
export const gridCols = createMemo(() => 
  params.targetWidth > 0 
    ? Math.floor(params.targetWidth / params.cellSize)
    : Math.floor(canvasWidth / params.cellSize)
);
```

### Font Atlas Data Structure

```rust
// Generated from unscii-8.hex (16 density-ordered glyphs)
pub struct FontAtlas {
    pub width: u32,        // 128 (16 glyphs × 8px)
    pub height: u32,       // 8
    pub glyph_width: u32,  // 8
    pub glyph_height: u32, // 8
    pub glyph_count: u32,  // 16
    pub data: Vec<u8>,     // RGBA: 128×8×4 = 4096 bytes
                           // White (255) = ink, Black (0) = background
    pub char_ramp: [char; 16],  // [' ', '.', ':', '-', '=', '+', '*', '#', '%', '@', '8', '&', 'W', 'M', '#', '█']
}
```

---

## Build & Deploy Pipeline

### Development
```bash
# Terminal 1: WASM rebuild on change
cd core && cargo watch -x "wasm-pack build --target web --out-dir ../frontend/pkg --dev"

# Terminal 2: Vite dev server
cd frontend && npm run dev
```

### Production Build
```bash
# 1. Build optimized WASM
cd core && wasm-pack build --release --target web --out-dir ../frontend/pkg

# 2. Run wasm-opt for additional size reduction
wasm-opt -Oz --enable-bulk-memory --enable-simd --strip-all \
  -o frontend/pkg/unscii_converter_core_bg.wasm \
  frontend/pkg/unscii_converter_core_bg.wasm

# 3. Build frontend
cd frontend && npm run build

# 4. Deploy dist/ to GitHub Pages / Netlify
```

### Vite Config (frontend/vite.config.ts)
```typescript
import { defineConfig } from 'vite';
import solidPlugin from 'vite-plugin-solid';
import wasm from 'vite-plugin-wasm';

export default defineConfig({
  plugins: [wasm(), solidPlugin()],
  optimizeDeps: {
    exclude: ['unscii-converter-core'],
  },
  build: {
    target: 'esnext',
    minify: 'esbuild',
  },
  server: {
    headers: {
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp',
    },
  },
});
```

---

## Performance Budgets

| Metric | Target |
|--------|--------|
| WASM bundle (gzipped) | < 500 KB |
| Initial load (total) | < 1 MB |
| Frame time (1080p) | < 16.67 ms (60 fps) |
| Convert latency (1MP) | < 100 ms |
| Memory (peak) | < 100 MB |

---

## Error Handling

| Layer | Strategy |
|-------|----------|
| WASM | Return `Result<T, JsValue>`; panic = abort (smaller binary) |
| WebGL | Check `getError()` after each call; fallback to CPU path |
| SolidJS | Error boundaries per component; toast notifications |
| Export | Validate inputs; return `Result<Blob, Error>` |

---

## Security

- **No server** — fully client-side
- **No user data leaves browser** — all processing local
- **WASM sandbox** — no filesystem/network access
- **CSP** — `default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; img-src 'self' data: blob:;`

---

## Future Extensibility

| Extension Point | Mechanism |
|-----------------|-----------|
| New font variants | Add .hex to public/fonts/, register in variants.rs |
| New dither algorithms | Implement `DitherAlgorithm` trait in dither/ |
| New export formats | Add function to export/ + WASM export |
| New CRT effects | Add uniform + shader code in postprocess.wgsl |
| Batch processing | Add `convert_batch()` accepting multiple images |

---

*This architecture is derived from the research in `./research/` and implements the requirements in `./goal-and-requirements.md`.*