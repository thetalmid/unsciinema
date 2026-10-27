# Research Directory Index

## Overview
Comprehensive research for the **Unscii Image Converter** — a Rust/WASM + SolidJS v2 web app for converting images to ASCII art using the unscii font, with real-time artistic controls.

**Last Updated:** October 27, 2026  
**Target Stack:** Rust → WASM (wasm-pack) + SolidJS v2 (Vite) + WebGL (WGSL) + Static Hosting

---

## Directory Structure

```
research/
├── unscii-font/
│   └── FORMAT_SPEC.md          # unscii .hex format specification, parsing, variants
├── wasm-toolchain/
│   └── WASM_PACK_VITE_SETUP.md # wasm-pack + Vite + SolidJS v2 project setup
├── webgl-ascii/
│   └── WEBGL_ASCII_SHADER.md   # WebGL/WGSL compute shader for real-time ASCII rendering
├── webgl-dithering/
│   └── WEBGL_DITHERING.md      # Dithering algorithms (CPU + GPU implementations)
├── image-processing/
│   └── IMAGE_PROCESSING_WASM.md # Rust image crate, zenwebp, gifski-wasm for WASM
├── export-formats/
│   └── EXPORT_FORMATS.md       # PNG, WebP, GIF, Text, HTML, ANSI export implementations
├── reference-projects/
│   └── REFERENCE_PROJECTS.md   # Analysis of 9 existing ASCII converters
├── wasm-optimization/
│   └── WASM_OPTIMIZATION.md    # Bundle size optimization (<500KB gzipped)
└── README.md                   # This file
```

---

## Key Technical Decisions

| Area | Decision | Rationale |
|------|----------|-----------|
| **Font** | unscii .hex format (8×8, 8×16) | Public domain, bitmap, multiple styles |
| **WASM Build** | wasm-pack + Vite + `vite-plugin-wasm` | Standard, well-documented, SolidJS compatible |
| **UI Framework** | SolidJS v2 | Fine-grained reactivity, no VDOM, small bundle |
| **ASCII Rendering** | WebGL compute shader + font atlas | 60fps real-time, GPU parallel, crisp glyphs |
| **Dithering** | Hybrid: CPU (error diffusion) + GPU (ordered) | Quality + performance balance |
| **WebP** | zenwebp (pure Rust) | WASM SIMD128, no C deps, lossy+lossless |
| **GIF** | gifski-wasm (JS) | High quality, multi-threaded, proven |
| **Export** | PNG, WebP, GIF, Text, HTML, ANSI | Comprehensive output options |
| **Hosting** | Static (GitHub Pages/Netlify) | 100% client-side, free, no server |
| **Bundle Target** | < 500 KB gzipped | Fast load, good UX |

---

## Module Architecture (Planned)

```
unscii-converter/
├── core/                    # Rust crate → WASM
│   ├── src/
│   │   ├── lib.rs           # WASM exports
│   │   ├── font/            # unscii .hex parsing, atlas generation
│   │   ├── image/           # Loading, processing, luminance
│   │   ├── dither/          # Floyd-Steinberg, Atkinson, Bayer, etc.
│   │   ├── convert/         # Image → ASCII grid mapping
│   │   └── export/          # PNG, WebP, Text, HTML encoders
│   ├── Cargo.toml
│   └── .rsw.json
├── frontend/                # SolidJS + Vite
│   ├── src/
│   │   ├── index.tsx        # Mount
│   │   ├── App.tsx          # Root
│   │   ├── components/
│   │   │   ├── Canvas.tsx   # WebGL canvas + render loop
│   │   │   ├── ControlPanel.tsx # Sliders, inputs
│   │   │   ├── FileDrop.tsx # Drag-and-drop
│   │   │   └── ExportModal.tsx # Export options
│   │   ├── hooks/
│   │   │   ├── useUnscii.ts   # WASM initialization
│   │   │   ├── useWebGL.ts    # WebGL context, shaders
│   │   │   └── useExport.ts   # Download helpers
│   │   └── shaders/
│   │       ├── ascii.wgsl     # ASCII compute shader
│   │       ├── dither.wgsl    # Ordered dithering shader
│   │       └── luma.wgsl      # BT.709 luminance
│   ├── index.html
│   ├── vite.config.ts
│   └── package.json
├── public/
│   └── fonts/
│       ├── unscii-8.hex
│       ├── unscii-16.hex
│       └── unscii-8.woff    # For HTML export
├── research/                # This directory
└── README.md
```

---

## Data Flow

```
User drops image
       │
       ▼
┌──────────────────┐
│  WASM: decode    │  image crate (PNG/JPEG/GIF/WebP)
│  image → RGBA    │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│  WASM: luminance │  BT.709 (0.2126, 0.7152, 0.0722)
│  RGBA → Luma     │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│  WASM: dither/   │  Floyd-Steinberg, Atkinson (CPU)
│  threshold       │  Bayer, Blue Noise (GPU shader)
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│  WASM: map to    │  Density → glyph index (0-15)
│  unscii glyphs   │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│  GPU: WebGL      │  Compute shader
│  render ASCII    │  Font atlas (128×8) lookup
│  (real-time)     │  Strength blend
└────────┬─────────┘
         │
         ▼
    Canvas display
         │
    ┌────┴────┐
    ▼         ▼
Export    Export
(PNG/     (Text/
 WebP/    HTML/
 GIF)     ANSI)
```

---

## Interactive Controls (All Real-Time)

| Parameter | Type | Range | Affects |
|-----------|------|-------|---------|
| **Dither Algorithm** | Select | none, floyd-steinberg, atkinson, bayer4, bayer8, blue-noise, sierra, jjn, stucki, burkes | Dithering method |
| **Dither Strength** | Slider | 0.0 – 1.0 | Blend original ↔ dithered |
| **Threshold** | Slider | 0.0 – 1.0 | Global binarization |
| **Character Set** | Select | unscii-8, unscii-8-alt, fantasy, mcr, tall, thin, unscii-16 | Font variant |
| **Cell Size / Scale** | Slider | 4 – 32 px | Output resolution |
| **Color Mode** | Select | mono, color, palette | Output coloring |
| **Palette** | Select + Custom | 31 presets + 2-6 colors | Color mapping |
| **Alpha Mode** | Select | ignore, transparent, blend | Alpha channel handling |
| **Contrast** | Slider | 0.1 – 3.0 | Pre-processing |
| **Gamma** | Slider | 0.5 – 2.5 | Pre-processing |
| **CRT Effects** | Toggles | scanlines, curvature, bloom, vignette | Post-processing |

---

## Implementation Priority

### Phase 1: Core WASM Module
1. [ ] Project setup (wasm-pack + Vite + SolidJS)
2. [ ] Image loading (PNG, JPEG, GIF, WebP)
3. [ ] unscii .hex font parsing + atlas generation
4. [ ] Luminance conversion (BT.709)
5. [ ] Basic threshold mapping → ASCII grid

### Phase 2: WebGL Renderer
1. [ ] WebGL2 context + WGSL compute shader pipeline
2. [ ] Font atlas texture upload
3. [ ] ASCII rendering shader (from bdip-core)
4. [ ] Real-time parameter uniforms
5. [ ] Strength blend (0.0 = original image)

### Phase 3: Dithering
1. [ ] CPU: Floyd-Steinberg, Atkinson in Rust
2. [ ] GPU: Bayer 4×4, 8×8, Blue Noise in WGSL
3. [ ] Palette mapping (31 presets + custom)

### Phase 4: SolidJS UI
1. [ ] Control panel with all sliders/selects
2. [ ] File drag-and-drop + preview
3. [ ] WebGL canvas integration
4. [ ] Export modal (PNG, WebP, GIF, Text, HTML, ANSI)

### Phase 5: Polish
1. [ ] Download unscii font files
2. [ ] WASM bundle optimization (< 500 KB gzipped)
3. [ ] Deploy to GitHub Pages / Netlify
4. [ ] Documentation, README

---

## References Quick Links

### Specifications
- [unscii font](http://viznut.fi/unscii/) | [GitHub](https://github.com/viznut/unscii)
- [Unifont HEX format](https://unifoundry.com/unifont.html)

### Toolchain
- [wasm-pack](https://github.com/wasm-bindgen/wasm-pack)
- [vite-plugin-wasm](https://github.com/Menci/vite-plugin-wasm)
- [vite-plugin-solid](https://github.com/solidjs/vite-plugin-solid)
- [SolidJS v2](https://www.solidjs.com/blog/solid-2-0-rc-the-big-reveal)

### WebGL / Shaders
- [bdip-core ASCII shader](https://docs.rs/crate/bdip-core/latest/source/shaders-wip/shaders/ascii_art/ascii_art_ascii.wgsl)
- [Efecto article](https://tympanus.net/codrops/2026/01/04/efecto-building-real-time-ascii-and-dithering-effects-with-webgl-shaders/)
- [Efecto live](https://efecto.app/)

### Image Processing
- [image crate](https://crates.io/crates/image)
- [zenwebp](https://github.com/imazen/zenwebp) (Pure Rust WebP)
- [gifski-wasm](https://github.com/jamsinclair/gifski-wasm)

### Optimization
- [Rust WASM Code Size](https://rustwasm.github.io/docs/book/reference/code-size.html)
- [wasm-opt](https://github.com/WebAssembly/binaryen)

### Reference Projects
- [lecepin/rust-wasm-image-ascii](https://github.com/lecepin/rust-wasm-image-ascii)
- [0xataru/ascii-rs](https://github.com/0xataru/ascii-rs)
- [beamterm](https://github.com/junkdog/beamterm)
- [ferroterm](https://github.com/DatanoiseTV/ferroterm)

---

## Next Steps

1. **Create `architecture.md`** — Formalize the module breakdown, interfaces, and data flow
2. **Initialize project** — `cargo new --lib core` + `npm create vite@latest frontend -- --template solid-ts`
3. **Implement core WASM module** — Start with font parsing + image loading
4. **Build WebGL shader pipeline** — Port bdip-core ASCII shader to project
5. **Wire SolidJS UI** — Reactive controls → WASM → WebGL → Canvas

---

*This research is current as of October 27, 2026. All links verified accessible.*