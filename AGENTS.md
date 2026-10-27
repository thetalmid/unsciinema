# AGENTS.md — Unscii Image Converter

## Quick Start for Agents

**Project:** Rust/WASM + SolidJS v2 web app for converting images to unscii ASCII art with real-time artistic controls. 100% client-side, static hosting.

**Workspace:** `C:/Users/marcelo.sampaio/.thinkrail/worktrees/cuzko/workspace-1`

---

## Research (READ THIS FIRST)

**Directory:** `./research/`  
**Index:** `./research/README.md` — Comprehensive overview of all 8 research files.

| Need | Read |
|------|------|
| unscii font format / parsing | `research/unscii-font/FORMAT_SPEC.md` |
| Project setup (wasm-pack + Vite + SolidJS) | `research/wasm-toolchain/WASM_PACK_VITE_SETUP.md` |
| WebGL ASCII shader / font atlas | `research/webgl-ascii/WEBGL_ASCII_SHADER.md` |
| Dithering algorithms (CPU + GPU) | `research/webgl-dithering/WEBGL_DITHERING.md` |
| Image loading / WebP / GIF in WASM | `research/image-processing/IMAGE_PROCESSING_WASM.md` |
| Export formats (PNG/WebP/GIF/Text/HTML/ANSI) | `research/export-formats/EXPORT_FORMATS.md` |
| Reference project analysis | `research/reference-projects/REFERENCE_PROJECTS.md` |
| WASM bundle optimization (<500KB gz) | `research/wasm-optimization/WASM_OPTIMIZATION.md` |

---

## Key Decisions (Do Not Re-Research)

| Area | Decision |
|------|----------|
| Font | unscii .hex (8×8, 8×16), Public Domain |
| Build | wasm-pack + Vite + `vite-plugin-wasm` |
| UI Framework | SolidJS v2 (fine-grained reactivity) |
| ASCII Rendering | WebGL compute shader + 128×8 font atlas |
| Dithering | Hybrid: CPU (Floyd-Steinberg, Atkinson) + GPU (Bayer, Blue Noise) |
| WebP | zenwebp (pure Rust, WASM SIMD128) |
| GIF | gifski-wasm (JS side, multi-threaded) |
| Exports | PNG, WebP, GIF, Text, HTML, ANSI |
| Bundle Target | < 500 KB gzipped |
| Hosting | Static (GitHub Pages / Netlify) |

---

## Current Status

**Spec:** `goal-and-requirements.md` (id: `goal-unscii-converter`, status: `active`)  
**Plan:** See todo list (`todo_list` tool)  
**Phase:** Architecture → Core WASM Module → WebGL Renderer → SolidJS UI → Polish

---

## Commands

```bash
# Build WASM (from core/)
wasm-pack build --release --target web --out-dir ../frontend/pkg

# Dev server (from frontend/)
npm run dev

# Production build
npm run build

# Check WASM size
gzip -c frontend/pkg/*_bg.wasm | wc -c
```

---

## Architecture (One-Liner)

```
WASM: decode → luminance → dither/threshold → map to unscii glyphs → ASCII grid
         │
         ▼
GPU (WGSL compute): font atlas lookup → real-time render → canvas
         │
         ▼
Exports: PNG/WebP/GIF (Rust) + Text/HTML/ANSI (Rust/JS)
```

---

*For detailed research, see `./research/README.md`. For product spec, see `./goal-and-requirements.md`.*