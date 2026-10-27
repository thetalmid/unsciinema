# wasm-pack + Vite + SolidJS v2 Setup

## Overview
Modern Rust→WASM pipeline using wasm-pack for compilation and Vite for bundling, with SolidJS v2 as the reactive UI framework.

## Project Structure
```
project-root/
├── Cargo.toml              # Rust crate config
├── src/
│   └── lib.rs              # #[wasm_bindgen] exports
├── .rsw.json               # rsw config (optional, for solidjs-vite-wasm)
├── vite.config.ts          # Vite + wasm plugin config
├── package.json            # npm deps + scripts
├── index.html              # Entry HTML
├── src/
│   ├── index.tsx           # SolidJS mount
│   ├── App.tsx             # Root component
│   └── components/         # UI components
└── public/
    └── fonts/              # Static assets (unscii .hex files)
```

## Cargo.toml (Rust crate)
```toml
[package]
name = "unscii-converter-core"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
wasm-bindgen = "0.2"
web-sys = { version = "0.3", features = ["HtmlCanvasElement", "WebGl2RenderingContext", "ImageData", "FileReader", "Blob"] }
js-sys = "0.3"
image = { version = "0.25", features = ["png", "jpeg", "gif", "webp", "bmp"] }
zenwebp = "0.4"  # Pure Rust WebP for WASM
gifski-wasm = "2.2"  # or gifski-lite for GIF
console_error_panic_hook = "0.1"
getrandom = { version = "0.2", features = ["js"] }

[profile.release]
opt-level = "z"        # Optimize for size
lto = true             # Link-time optimization
codegen-units = 1      # Single codegen unit for better optimization
panic = "abort"        # Smaller than unwind
strip = true           # Strip symbols
```

## vite.config.ts
```typescript
import { defineConfig } from 'vite'
import solidPlugin from 'vite-plugin-solid'
import wasm from 'vite-plugin-wasm'

export default defineConfig({
  plugins: [
    wasm(),  // Must come before solidPlugin
    solidPlugin(),
  ],
  optimizeDeps: {
    exclude: ['unscii-converter-core', 'gifski-wasm'],  // WASM modules
  },
  build: {
    target: 'esnext',
    minify: 'esbuild',
  },
  server: {
    headers: {
      // Required for SharedArrayBuffer (multi-threaded WASM)
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp',
    },
  },
})
```

## package.json
```json
{
  "name": "unscii-converter",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "npm run build:wasm && vite build",
    "build:wasm": "wasm-pack build --target web --out-dir pkg --release",
    "preview": "vite preview"
  },
  "dependencies": {
    "solid-js": "^1.9.0",
    "unscii-converter-core": "file:./pkg"
  },
  "devDependencies": {
    "vite": "^5.0.0",
    "vite-plugin-solid": "^2.10.0",
    "vite-plugin-wasm": "^3.3.0",
    "wasm-pack": "^0.12.0"
  }
}
```

## Rust lib.rs (WASM exports)
```rust
use wasm_bindgen::prelude::*;
use image::{DynamicImage, GenericImageView};
use zenwebp::{EncodeRequest, LossyConfig, PixelLayout};

#[wasm_bindgen]
pub struct UnsciiConverter {
    // Font data loaded at init
    font_atlas: Vec<u8>,  // 128x8 texture atlas for 16 chars × 8px
}

#[wasm_bindgen]
impl UnsciiConverter {
    #[wasm_bindgen(constructor)]
    pub fn new(font_hex: &str) -> Result<UnsciiConverter, JsValue> {
        // Parse unscii .hex format, generate 8x8 atlas
        Ok(Self { font_atlas: generate_atlas(font_hex) })
    }

    #[wasm_bindgen]
    pub fn convert_image(&self, image_data: &[u8], params: &ConvertParams) -> Result<Vec<u8>, JsValue> {
        // 1. Decode image (PNG/JPEG/GIF/WebP)
        // 2. Apply dithering/threshold in Rust
        // 3. Map to unscii glyphs
        // 4. Return RGBA bytes for WebGL texture upload
    }

    #[wasm_bindgen]
    pub fn export_png(&self, width: u32, height: u32, data: &[u8]) -> Vec<u8> { ... }
    #[wasm_bindgen]
    pub fn export_webp(&self, width: u32, height: u32, data: &[u8], quality: f32) -> Vec<u8> { ... }
    #[wasm_bindgen]
    pub fn export_gif(&self, frames: &[Vec<u8>], width: u32, height: u32, fps: u32) -> Vec<u8> { ... }
    #[wasm_bindgen]
    pub fn export_text(&self, ascii_grid: &[u16], cols: u32, rows: u32) -> String { ... }
    #[wasm_bindgen]
    pub fn export_html(&self, ascii_grid: &[u16], cols: u32, rows: u32, fg: &str, bg: &str) -> String { ... }
}

#[wasm_bindgen]
pub struct ConvertParams {
    pub cell_size: f32,
    pub dither_mode: u32,      // 0=none, 1=FloydSteinberg, 2=Atkinson, 3=Bayer4, 4=Bayer8
    pub threshold: f32,
    pub charset: u32,          // Font variant index
    pub scale: f32,
    pub color_mode: u32,       // 0=mono, 1=color, 2=palette
    pub palette: Vec<u32>,     // Packed RGBA palette
    pub alpha_mode: u32,       // 0=ignore, 1=transparent, 2=blend
}
```

## SolidJS Integration Pattern
```typescript
// src/hooks/useUnscii.ts
import { createSignal, onMount, onCleanup } from 'solid-js'
import init, { UnsciiConverter, ConvertParams } from 'unscii-converter-core'

export function useUnscii() {
  const [converter, setConverter] = createSignal<UnsciiConverter | null>(null)
  const [ready, setReady] = createSignal(false)

  onMount(async () => {
    await init()
    const fontResp = await fetch('/fonts/unscii-8.hex')
    const fontHex = await fontResp.text()
    const conv = new UnsciiConverter(fontHex)
    setConverter(conv)
    setReady(true)
  })

  return { converter, ready }
}
```

## Key References
- **solidjs-vite-wasm template**: https://github.com/karan-vk/solidjs-vite-wasm
- **solid-markdown-wasm**: https://github.com/zeon256/solid-markdown-wasm
- **wasm-pack Vite integration**: https://www.webassembly-wasm.com/compilation-pipelines-toolchain-setup/esm-bindings-module-generation/bundling-wasm-esm-with-vite/
- **wasm-bindgen guide**: https://rustwasm.github.io/docs/wasm-bindgen/