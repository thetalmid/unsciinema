# Reference Projects Analysis

## Overview
Analysis of existing Rust/WASM ASCII art converters and related projects for architectural patterns and implementation ideas.

---

## 1. lecepin/rust-wasm-image-ascii
**URL:** https://github.com/lecepin/rust-wasm-image-ascii  
**Live:** https://lecepin.github.io/rust-wasm-image-ascii/  
**Tech:** Rust → WASM, vanilla JS frontend

### Architecture
```
src/
├── lib.rs              # WASM exports
├── utils.rs            # Image loading helpers
└── tai/                # Ported from tai (MustafaSalih1993/tai)
    ├── operations/
    │   ├── ascii.rs    # Standard ASCII
    │   ├── braille.rs  # Braille patterns
    │   ├── dither.rs   # Dithering algorithms
    │   ├── onechar.rs  # Single char mode
    │   └── otsu_threshold.rs
    └── arguments/      # Config parsing
```

### Key Exports
```rust
#[wasm_bindgen]
pub fn get_gray_image(img_data: &[u8]) -> Vec<u8> { ... }

#[wasm_bindgen]
pub fn get_ascii_by_image(img_data: &[u8], config: &str) -> String { ... }

#[wasm_bindgen]
pub fn get_ascii_by_image_tai(img_data: &[u8], config: &str) -> String { ... }
```

### Luminance Formula
```rust
// BT.709 standard
pub fn get_luminance(r: u8, g: u8, b: u8) -> f32 {
    0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32
}
```

### Character Ramps
- **Standard:** ` .:-=+*#%@`
- **Braille:** 8-dot patterns (⠁⠂⠄...)
- **Blocks:** ▁▂▃▄▅▆▇█
- **Tai modes:** Multiple densities

### Strengths
- Clean WASM exports
- Multiple output modes (ASCII, Braille, blocks, Tai)
- Dithering implementations (Floyd-Steinberg, Atkinson, ordered)
- Otsu thresholding for adaptive binarization
- Published to npm: `rust-wasm-image-ascii`

### Limitations
- No unscii font support
- No real-time WebGL preview (CPU-only)
- Vanilla JS frontend (no reactive framework)
- No color output
- No export formats beyond text

---

## 2. 0xataru/ascii-rs
**URL:** https://github.com/0xataru/ascii-rs  
**Tech:** Rust (Axum backend) + Yew (WASM frontend) + Clean Architecture

### Architecture (Clean Architecture)
```
src/
├── domain/
│   ├── entities/       # ImageData, AsciiArt
│   ├── repositories/   # Traits
│   └── value_objects/  # ConversionConfig, ImageFormat
├── application/
│   ├── use_cases/      # ConvertImageToAscii, UploadImage
│   └── services/       # AsciiConversionService
├── infrastructure/
│   ├── repositories/   # In-memory impls
│   └── web/            # Axum routes, extractors
└── presentation/
    └── handlers/       # HTTP handlers

frontend/               # Yew WASM app
├── src/lib.rs
├── Cargo.toml
└── Trunk.toml
```

### Key Features
- REST API (`POST /api/upload`, `POST /api/convert/{id}`)
- Clean Architecture with dependency injection
- Yew frontend with Trunk bundler
- Algorithm improvements:
  - Catmull-Rom resampling
  - Adaptive contrast enhancement
  - Gaussian blur preprocessing
  - Adaptive thresholding (histogram equalization)
  - Gamma correction
  - Perceptual ASCII mapping

### API Parameters
```
POST /api/convert/{image_id}?width=100&detail=high&contrast=1.2&blur=0.5
```

### Strengths
- Production-grade architecture
- Well-documented API
- Multiple algorithm improvements
- Docker deployment ready
- Health checks, monitoring

### Limitations
- Backend required (not fully client-side)
- Yew framework (not SolidJS)
- No unscii font
- No WebGL real-time preview
- Server-side processing

---

## 3. maixnor/ascii
**URL:** https://github.com/maixnor/ascii  
**Tech:** Rust + egui → WASM

### Notes
- Native app running in browser via WASM
- Uses egui (immediate mode GUI)
- Less relevant for SolidJS approach

---

## 4. rikdgd/image2ascii-web
**URL:** https://github.com/rikdgd/image2ascii-web  
**Live:** https://rikdgd.github.io/image2ascii-web/  
**Tech:** Rust → WASM + vanilla HTML/CSS/JS

### Simplicity
- Very simple frontend
- Rust conversion logic only
- Good reference for minimal WASM integration

---

## 5. lecepin/rust-wasm-image-ascii (Tai version)
Ports the **tai** project (https://github.com/MustafaSalih1993/tai) to WASM.

### Tai Features
- Multiple ASCII styles
- Braille rendering
- Dithering algorithms
- Otsu thresholding
- One-character mode

---

## 6. arscii (crates.io)
**URL:** https://docs.rs/arscii/latest/  
**Note:** Simple ASCII renderer, CLI-focused

---

## 7. @dotart/ui (npm)
**URL:** https://www.npmjs.com/package/@dotart/ui  
**Note:** Browser-native ASCII UI engine, commercial

---

## 8. Efecto (Reference for WebGL)
**URL:** https://efecto.app/  
**Article:** https://tympanus.net/codrops/2026/01/04/efecto-building-real-time-ascii-and-dithering-effects-with-webgl-shaders/

### Key Techniques (Not Rust, but algorithmic reference)
- **ASCII on GPU** (WebGL shader with procedural character drawing)
- **Dithering on CPU** (error diffusion is sequential)
- **Bloom on GPU** (WebGPU/WebGL)
- **CRT effects** (scanlines, curvature, chromatic aberration, vignette)
- **8 ASCII styles** (standard, dense, minimal, blocks, braille, technical, matrix, hatching)
- **31 color palettes** + custom
- **Luminance weights:** BT.709 (0.299, 0.587, 0.114) — note: article uses BT.601, but BT.709 is more accurate for sRGB

### Shader Character Generation
```glsl
// Procedural 5×7 character drawing in GLSL
// Each character = function returning 1.0 (ink) or 0.0 (background)
// Example: asterisk
bool center = (grid.x == 2.0 && grid.y == 3.0);
bool vert = (grid.x == 2.0 && (grid.y >= 2.0 && grid.y <= 4.0));
bool horiz = (grid.y == 3.0 && (grid.x >= 1.0 && grid.x <= 3.0));
bool diag1 = ((grid.x == 1.0 && grid.y == 2.0) || (grid.x == 3.0 && grid.y == 4.0));
bool diag2 = ((grid.x == 1.0 && grid.y == 4.0) || (grid.x == 3.0 && grid.y == 2.0));
return (center || vert || horiz || diag1 || diag2) ? 1.0 : 0.0;
```

### Our Difference
- We use **unscii font atlas** (bitmap font) instead of procedural characters
- We use **SolidJS** for UI (reactive, fine-grained)
- We target **100% client-side** (static hosting)

---

## 9. ferroterm / beamterm / doryen-rs (Terminal Renderers)
**URLs:** 
- https://github.com/DatanoiseTV/ferroterm
- https://github.com/junkdog/beamterm
- https://jice-nospam.github.io/doryen-rs/

### Relevance
- GPU-accelerated terminal rendering
- Font atlas techniques
- WebGL + Rust/WASM
- Single draw call via instancing
- Dynamic glyph atlases

### beamterm Highlights
- Single draw call for entire terminal (200×80 cells)
- Instanced rendering
- Flexible font atlases
- WebGL2 (WASM) + OpenGL 3.3 (native) via `glow`
- Subcell resolution

---

## Summary: What to Adopt vs. Differentiate

| Feature | Adopt From | Our Approach |
|---------|------------|--------------|
| WASM image loading | lecepin, 0xataru | `image` crate + zenwebp |
| Dithering algorithms | lecepin, Efecto | CPU (error diffusion) + GPU (ordered) |
| Clean Architecture | 0xataru | Simplified module design |
| WebGL ASCII shader | Efecto, bdip-core | Font atlas + compute shader |
| Font atlas rendering | beamterm, ferroterm | Static unscii atlas (128×8) |
| Reactive UI | 0xataru (Yew) | **SolidJS v2** (fine-grained) |
| Build system | 0xataru (Trunk) | **wasm-pack + Vite** |
| Export formats | 0xataru (API) | **Client-side: PNG, WebP, GIF, Text, HTML** |
| unscii font | **Unique** | Parse .hex, generate atlas |
| Real-time preview | Efecto (WebGL) | **WebGL compute shader** |
| Artistic controls | Efecto (31 palettes) | **Custom: dithering, threshold, charset, scale, color mapping, alpha** |
| Deployment | 0xataru (Docker) | **Static: GitHub Pages/Netlify** |

---

## Key Implementation Insights

1. **Separate CPU/GPU work**: Error diffusion dithering → CPU (WASM); Ordered dithering, ASCII rendering → GPU (WebGL)

2. **Font atlas > Procedural**: unscii provides consistent, high-quality glyphs; procedural is flexible but harder to match font aesthetics

3. **WASM for heavy lifting**: Image decode, dithering, font parsing, export encoding — all in Rust/WASM

4. **SolidJS for UI**: Fine-grained reactivity perfect for slider-driven real-time updates

5. **Static hosting**: 100% client-side = free hosting, no server costs

6. **WASM bundle size**: Target < 500KB gzipped; use `opt-level = "z"`, `wasm-opt`, `zenwebp` (pure Rust)