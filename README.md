# Unscii Image Converter

Interactive Rust/WASM + SolidJS web app for converting images to ASCII art using the unscii font, with real-time artistic controls.

## Features

- **Real-time preview** - WebGL2 compute shader rendering at 60fps
- **unscii font support** - 7 variants (8×8, 8×16, alt, fantasy, mcr, tall, thin)
- **10 dithering algorithms** - Floyd-Steinberg, Atkinson, Sierra, JJN, Stucki, Burkes (CPU) + Bayer 4×4, Bayer 8×8, Blue Noise (GPU)
- **31 color palettes** - Terminal, GameBoy, Synthwave, Cyberpunk, and more + custom
- **Full artistic control** - Cell size, threshold, contrast, gamma, alpha handling, CRT effects
- **Multiple exports** - PNG, WebP, GIF, Text, HTML, ANSI
- **100% client-side** - No server, free static hosting

## Stack

- **Core**: Rust → WebAssembly (wasm-pack)
- **UI**: SolidJS v2 + Vite
- **Rendering**: WebGL2 / WGSL compute shaders
- **Image**: `image` crate + `zenwebp` (pure Rust WebP)
- **Fonts**: unscii .hex format (Public Domain)

## Project Structure

```
unscii-converter/
├── core/                    # Rust crate → WASM
│   ├── src/
│   │   ├── font/           # unscii .hex parsing, atlas generation
│   │   ├── image/          # Loading, luminance, preprocessing
│   │   ├── dither/         # 10 dithering algorithms
│   │   ├── convert/        # Image → ASCII grid mapping
│   │   └── export/         # PNG, WebP, Text, HTML, ANSI
│   └── Cargo.toml
├── frontend/                # SolidJS + Vite
│   ├── src/
│   │   ├── components/     # Canvas, ControlPanel, FileDrop, ExportModal
│   │   ├── hooks/          # useUnscii, useImage, useWebGL, useExport
│   │   ├── stores/         # Reactive parameters
│   │   └── shaders/        # WGSL shaders
│   └── package.json
├── public/fonts/           # unscii .hex font files
├── research/               # Technical research (8 topics)
├── architecture.md         # Module breakdown & interfaces
├── goal-and-requirements.md
└── AGENTS.md
```

## Development

### Prerequisites
- Rust 1.75+ with `wasm32-unknown-unknown` target
- Node.js 18+ with npm/pnpm
- `wasm-pack` (`cargo install wasm-pack`)

### Setup
```bash
# Build WASM module
cd core && wasm-pack build --release --target web --out-dir ../frontend/pkg

# Install frontend deps
cd frontend && npm install

# Start dev server (rebuilds WASM on change)
npm run dev
```

### Production Build
```bash
# Build optimized WASM
cd core && wasm-pack build --release --target web --out-dir ../frontend/pkg

# Optional: further optimize with wasm-opt
wasm-opt -Oz --enable-bulk-memory --enable-simd --strip-all \
  -o frontend/pkg/unscii_converter_core_bg.wasm \
  frontend/pkg/unscii_converter_core_bg.wasm

# Build frontend
cd frontend && npm run build

# Deploy dist/ to GitHub Pages, Netlify, Vercel, etc.
```

## Interactive Controls

| Category | Parameters |
|----------|------------|
| **Dithering** | Algorithm (10), Strength (0-1), Threshold (0-1) |
| **Font** | Variant (7), Cell Size (4-32px) |
| **Rendering** | Color Mode (Mono/Color/Palette), Alpha Mode, Contrast, Gamma |
| **Color** | 31 presets + Custom (up to 6 colors) |
| **CRT Effects** | Scanlines, Curvature, Bloom, Vignette |

## Export Formats

| Format | Features |
|--------|----------|
| PNG | Lossless, transparency |
| WebP | Lossy/Lossless, transparency, animation |
| GIF | Animation (via gifski-wasm) |
| Text | Plain ASCII |
| HTML | Styled with embedded font |
| ANSI | 24-bit terminal codes |

## Research

See `research/README.md` for comprehensive documentation on:
- unscii font format specification
- wasm-pack + Vite + SolidJS setup
- WebGL ASCII shader (from bdip-core)
- Dithering algorithms (CPU + GPU)
- Image processing in WASM
- Export format implementations
- Reference project analysis
- WASM bundle optimization (<500KB gzipped)

## License

MIT OR Apache-2.0

## Credits

- **unscii font**: Viznut (Public Domain / CC-0)
- **ASCII shader approach**: bdip-core / Efecto
- **Dithering algorithms**: Historical papers + Efecto
- **WebP**: zenwebp by Imazen (pure Rust)
- **GIF**: gifski by Kornel Lesiński