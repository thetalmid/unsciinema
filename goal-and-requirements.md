---
id: goal-unscii-converter
type: goal-and-requirements
title: Unscii Image Converter — Rust/WASM GUI App
status: active
---

# Unscii Image Converter

**Tagline**: Interactive Rust/WASM web app for converting images to ASCII art using the unscii font, with real-time artistic controls.

## Overview

A personal Rust GUI application (compiled to WebAssembly, running in-browser) that converts common image formats (PNG, JPEG, GIF, etc.) into ASCII art using the unscii font format (viznut.fi/unscii). Features interactive artistic controls with meaningful parameters for comprehensive creative control (dithering, threshold, character set, scaling, color mapping, alpha channel handling). Built as a learning project to explore Rust GUI development via WASM, Solid v2 for the UI, WebGL for performance-critical rendering, and image processing in Rust.

## Problem

Existing image-to-ASCII tools are primarily CLI-based with no visual preview, offer limited artistic control (fixed dithering, character sets, scaling), and lack proper unscii font format support. This makes iterative creative exploration slow — you can't see results in real-time while adjusting parameters, and the unscii font's unique characteristics aren't leveraged.

## Capabilities

1. **Load common image formats** (PNG, JPEG, GIF, etc.) via file dialog / drag-and-drop
2. **Real-time preview with interactive controls** — sliders/inputs for dithering, threshold, character set, scaling, color mapping, alpha channel handling — instant preview
3. **unscii font format support** — parse and render using .unscii font files from viznut.fi
4. **Export results** as PNG, WebP, GIF, plain text, or HTML

## Tech Notes

| Aspect | Choice | Rationale |
|--------|--------|-----------|
| Language | Rust → WebAssembly | Learning Rust; WASM gives near-native performance in browser |
| Frontend Framework | Solid v2 | Fine-grained reactivity, no virtual DOM, small bundle, great DX |
| Graphics/Rendering | WebGL | Hardware-accelerated dithering, font rendering, real-time preview at 60fps |
| Image Processing | Rust/WASM (image crate, webp/gif encoders) | Leverage Rust ecosystem; keep heavy lifting in WASM |
| Font Parsing | Rust/WASM | Parse .unscii format directly in WASM |
| Build System | wasm-pack + Vite | Standard Rust→WASM pipeline; fast dev server |
| Distribution | Static site (GitHub Pages / Netlify / Vercel) | 100% client-side — free hosting, no server costs |

---

*This spec is a living document and evolves with the project. Next step: sketch `architecture.md`, then build features through `brainstorming`.*