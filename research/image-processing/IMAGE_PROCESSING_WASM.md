# Rust Image Processing for WASM

## Overview
Using the `image` crate and related crates for image loading, processing, and encoding in WebAssembly. All crates must be `wasm32-unknown-unknown` compatible.

## Core Crates

### `image` crate (v0.25+)
Main imaging library. **Fully WASM compatible** with default features.

```toml
[dependencies]
image = { version = "0.25", features = ["png", "jpeg", "gif", "webp", "bmp", "tiff", "pnm", "qoi", "dds", "tga", "exr", "hdr", "avif"] }
```

**Supported formats in WASM:**
| Format | Decode | Encode | Notes |
|--------|--------|--------|-------|
| PNG | ✅ | ✅ | Full support |
| JPEG | ✅ | ✅ | Lossy only |
| GIF | ✅ | ✅ | Animation decode only |
| WebP | ✅ | ⚠️ | Decode yes, encode needs `webp` feature + libwebp (C) |
| BMP | ✅ | ✅ | |
| TIFF | ✅ | ❌ | Decode only |
| QOI | ✅ | ✅ | Fast, simple |
| AVIF | ✅ | ❌ | Needs `avif-native` (C libdav1d) |

### `zenwebp` — **Pure Rust WebP** (Recommended for WASM)
```toml
zenwebp = "0.4"
```
- **Pure Rust** — no C dependencies, `#![forbid(unsafe_code)]`
- **WASM SIMD128** accelerated via runtime CPU detection
- **Lossy (VP8) + Lossless (VP8L)** encode/decode
- **Animation** support
- **Alpha, ICC/EXIF/XMP** metadata
- MSRV: 1.93

```rust
use zenwebp::{EncodeRequest, LossyConfig, PixelLayout, oneshot::decode_rgba};

// Decode
let (rgba, w, h) = decode_rgba(webp_bytes)?;

// Encode lossy
let webp = EncodeRequest::lossy(
    &LossyConfig::new().with_quality(85.0).with_method(4),
    rgb_pixels, PixelLayout::Rgb8, width, height
).encode()?;

// Encode lossless
let webp = EncodeRequest::lossless(
    &LosslessConfig::new().with_quality(90.0).with_method(6),
    rgba_pixels, PixelLayout::Rgba8, width, height
).encode()?;

// Animation
use zenwebp::mux::AnimationEncoder;
let mut anim = AnimationEncoder::new();
anim.add_frame(rgba_frame1, width, height, 100)?;  // 100ms
anim.add_frame(rgba_frame2, width, height, 100)?;
let webp = anim.encode()?;
```

### `gifski-wasm` / `gifski-lite` — High-Quality GIF
```toml
gifski-wasm = "2.2"  # npm package, uses WASM
# OR
gifski-lite = "1.0"  # Rust crate, pngquant-based
```

**gifski-wasm (JavaScript/WASM):**
```javascript
import encode from 'gifski-wasm';

const gif = await encode({
  frames: [imageData1, imageData2, ...],  // Uint8Array or ImageData
  width: 800,
  height: 600,
  fps: 12,
  quality: 80,  // 1-100
  repeat: 0,    // 0 = infinite
});
```

**Requirements for multi-threading:**
- `Cross-Origin-Opener-Policy: same-origin`
- `Cross-Origin-Embedder-Policy: require-corp`
- Run in Web Worker

### `image-webp` — Pure Rust WebP for `image` crate
```toml
image-webp = "0.10"  # Integrates with image::codecs::webp
```
- Pure Rust decoder/encoder for `image` crate
- Supports lossless + lossy, animation, alpha
- Slower than zenwebp but integrates with `DynamicImage`

## Loading Images in WASM
```rust
use image::{ImageReader, DynamicImage, ImageFormat};
use std::io::Cursor;

#[wasm_bindgen]
pub fn load_image(bytes: &[u8]) -> Result<DynamicImage, JsValue> {
    let format = ImageFormat::from_path("dummy.png").unwrap_or(ImageFormat::Png);
    let img = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()?
        .decode()
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(img)
}

// Or detect format from bytes
pub fn detect_format(bytes: &[u8]) -> ImageFormat {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") { ImageFormat::Png }
    else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) { ImageFormat::Jpeg }
    else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") { ImageFormat::Gif }
    else if bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP" { ImageFormat::WebP }
    else { ImageFormat::Png }
}
```

## Converting to Working Buffers
```rust
fn image_to_rgba(img: &DynamicImage) -> (Vec<u8>, u32, u32) {
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    (rgba.into_raw(), w, h)
}

fn image_to_rgb(img: &DynamicImage) -> (Vec<u8>, u32, u32) {
    let rgb = img.to_rgb8();
    let (w, h) = rgb.dimensions();
    (rgb.into_raw(), w, h)
}
```

## Image Processing Operations

### Resize (Catmull-Rom recommended for ASCII)
```rust
use image::imageops::FilterType::CatmullRom;

let resized = img.resize(target_width, target_height, CatmullRom);
```

### Grayscale / Luminance
```rust
// BT.709 luminance
pub fn to_luminance(r: u8, g: u8, b: u8) -> u8 {
    let l = 0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32;
    l.clamp(0.0, 255.0) as u8
}

// Vectorized for SIMD
pub fn image_to_luma(img: &DynamicImage) -> Vec<u8> {
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let mut luma = vec![0u8; (w * h) as usize];
    for (i, pixel) in rgba.pixels().enumerate() {
        luma[i] = to_luminance(pixel[0], pixel[1], pixel[2]);
    }
    luma
}
```

### Contrast / Brightness / Gamma
```rust
pub fn adjust_contrast(pixels: &mut [u8], factor: f32) {
    let mid = 128.0;
    for p in pixels.iter_mut() {
        let v = *p as f32;
        *p = (mid + (v - mid) * factor).clamp(0.0, 255.0) as u8;
    }
}

pub fn adjust_gamma(pixels: &mut [u8], gamma: f32) {
    let inv_gamma = 1.0 / gamma;
    for p in pixels.iter_mut() {
        *p = (255.0 * (*p as f32 / 255.0).powf(inv_gamma)) as u8;
    }
}
```

### Gaussian Blur (separable, O(n) per pass)
```rust
pub fn gaussian_blur(input: &[u8], width: u32, height: u32, sigma: f32) -> Vec<u8> {
    let kernel = generate_gaussian_kernel(sigma);
    // Horizontal pass
    let mut temp = vec![0u8; input.len()];
    convolve_horizontal(input, &mut temp, width, height, &kernel);
    // Vertical pass
    let mut output = vec![0u8; input.len()];
    convolve_vertical(&temp, &mut output, width, height, &kernel);
    output
}
```

## WASM-Specific Considerations

### Memory Limits
- Default WASM memory: 1MB initial, grows to 2GB max
- Large images (e.g., 8000×8000 RGBA = 256MB) may hit limits
- Use `wasm-bindgen` with `--import-memory` for custom memory

### Performance Tips
```toml
[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

### SIMD in WASM
- `zenwebp` uses SIMD128 automatically on WASM
- `image` crate uses SIMD via `packed_simd` (nightly) or `std::simd` (stable 1.88+)
- Enable with `target-feature=+simd128` in `.cargo/config.toml`

### Threading (Rayon)
- `rayon` works in WASM with `wasm-bindgen-rayon`
- Requires SharedArrayBuffer headers (COOP/COEP)
- Use for parallel image processing (per-row, per-tile)

## Export Pipeline
```rust
#[wasm_bindgen]
pub fn export_png(rgba: &[u8], width: u32, height: u32) -> Vec<u8> {
    let mut buf = Vec::new();
    let mut encoder = png::Encoder::new(&mut buf, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(rgba).unwrap();
    buf
}

#[wasm_bindgen]
pub fn export_webp(rgba: &[u8], width: u32, height: u32, quality: f32) -> Vec<u8> {
    zenwebp::EncodeRequest::lossy(
        &zenwebp::LossyConfig::new().with_quality(quality).with_method(4),
        rgba, zenwebp::PixelLayout::Rgba8, width, height
    ).encode().unwrap()
}

#[wasm_bindgen]
pub fn export_gif(frames: &[Vec<u8>], width: u32, height: u32, fps: u32) -> Vec<u8> {
    // Use gifski-wasm from JS side, or gifski-lite from Rust
    // gifski-lite requires pngquant which may not compile to WASM easily
    // Best: export frames as RGBA from Rust, encode GIF in JS with gifski-wasm
}
```

## Alpha Channel Handling
```rust
// Premultiplied alpha for correct blending
fn premultiply_alpha(rgba: &mut [u8]) {
    for chunk in rgba.chunks_exact_mut(4) {
        let a = chunk[3] as f32 / 255.0;
        chunk[0] = (chunk[0] as f32 * a) as u8;
        chunk[1] = (chunk[1] as f32 * a) as u8;
        chunk[2] = (chunk[2] as f32 * a) as u8;
    }
}

// Unpremultiply for display
fn unpremultiply_alpha(rgba: &mut [u8]) {
    for chunk in rgba.chunks_exact_mut(4) {
        if chunk[3] > 0 {
            let inv_a = 255.0 / chunk[3] as f32;
            chunk[0] = (chunk[0] as f32 * inv_a).min(255.0) as u8;
            chunk[1] = (chunk[1] as f32 * inv_a).min(255.0) as u8;
            chunk[2] = (chunk[2] as f32 * inv_a).min(255.0) as u8;
        }
    }
}
```

## References
- **image crate**: https://crates.io/crates/image / https://docs.rs/image/latest/image/
- **zenwebp**: https://github.com/imazen/zenwebp (Pure Rust WebP, WASM SIMD128)
- **gifski-wasm**: https://github.com/jamsinclair/gifski-wasm
- **gifski-lite**: https://docs.rs/gifski-lite/latest/
- **image-webp**: https://github.com/image-rs/image-webp
- **webpx** (deprecated, use zenwebp): https://github.com/imazen/webpx