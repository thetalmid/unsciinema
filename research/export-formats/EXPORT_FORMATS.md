# Export Formats Implementation

## Overview
All export formats implemented in Rust/WASM for maximum performance, with JavaScript fallback for GIF (via gifski-wasm).

## Export Formats

### 1. PNG (Lossless, Transparency)
```rust
#[wasm_bindgen]
pub fn export_png(rgba: &[u8], width: u32, height: u32) -> Vec<u8> {
    let mut buf = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut buf, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_compression(png::Compression::Best);
        encoder.set_filter(png::FilterType::Paeth);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(rgba).unwrap();
    }
    buf
}
```
**Dependencies:** `png = "0.18"`

### 2. WebP (Lossy + Lossless, Transparency, Animation)
```rust
use zenwebp::{EncodeRequest, LossyConfig, LosslessConfig, PixelLayout};

#[wasm_bindgen]
pub fn export_webp(rgba: &[u8], width: u32, height: u32, quality: f32, lossless: bool) -> Vec<u8> {
    if lossless {
        EncodeRequest::lossless(
            &LosslessConfig::new().with_quality(quality).with_method(6),
            rgba, PixelLayout::Rgba8, width, height
        ).encode().unwrap()
    } else {
        EncodeRequest::lossy(
            &LossyConfig::new().with_quality(quality).with_method(4),
            rgba, PixelLayout::Rgba8, width, height
        ).encode().unwrap()
    }
}

#[wasm_bindgen]
pub fn export_webp_animation(frames: &[Vec<u8>], width: u32, height: u32, fps: u32, quality: f32) -> Vec<u8> {
    use zenwebp::mux::AnimationEncoder;
    let mut anim = AnimationEncoder::new();
    let frame_duration = 1000 / fps; // ms
    for frame in frames {
        anim.add_frame(frame, width, height, frame_duration).unwrap();
    }
    anim.encode().unwrap()
}
```
**Dependencies:** `zenwebp = "0.4"`

### 3. GIF (Animation, 256 colors/frame)
**Best approach:** Export frames from Rust, encode in JS with `gifski-wasm`

```rust
// Rust side: just provide RGBA frames
#[wasm_bindgen]
pub fn get_gif_frames(ascii_frames: &[Vec<u16>], width: u32, height: u32, palette: &[u32]) -> Vec<Vec<u8>> {
    ascii_frames.iter().map(|frame| {
        render_frame_to_rgba(frame, width, height, palette)
    }).collect()
}
```

```javascript
// JS side: use gifski-wasm
import encode from 'gifski-wasm';

const frames = await getGifFrames(asciiFrames, width, height, palette);
const gif = await encode({
  frames,
  width,
  height,
  fps: 12,
  quality: 80,
  repeat: 0,
});

// Trigger download
const blob = new Blob([gif], { type: 'image/gif' });
const url = URL.createObjectURL(blob);
const a = document.createElement('a');
a.href = url;
a.download = 'unscii-animation.gif';
a.click();
URL.revokeObjectURL(url);
```
**Dependencies:** `gifski-wasm` (npm)

### 4. Plain Text (.txt)
```rust
#[wasm_bindgen]
pub fn export_text(ascii_grid: &[u16], cols: u32, rows: u32) -> String {
    let mut out = String::with_capacity((cols + 1) as usize * rows as usize);
    for row in 0..rows {
        for col in 0..cols {
            let idx = ascii_grid[(row * cols + col) as usize] as usize;
            out.push(CHAR_RAMP[idx]);  // CHAR_RAMP = " .:-=+*#%@"
        }
        out.push('\n');
    }
    out
}
```

### 5. HTML (Styled, Copy-Paste Friendly)
```rust
#[wasm_bindgen]
pub fn export_html(
    ascii_grid: &[u16], 
    cols: u32, 
    rows: u32, 
    fg_color: &str,  // "#ffffff"
    bg_color: &str,  // "#000000"
    font_family: &str, // "'Unscii', monospace"
    line_height: f32, // 1.1
    font_size: u32,   // 12
) -> String {
    let mut html = String::new();
    html.push_str(&format!(r#"
<!DOCTYPE html>
<html>
<head>
<meta charset="UTF-8">
<title>Unscii ASCII Art</title>
<style>
body {{ background: {bg}; margin: 0; padding: 20px; font-family: {font}; }}
pre {{ 
    color: {fg}; 
    background: {bg}; 
    font-size: {fs}px; 
    line-height: {lh}; 
    white-space: pre; 
    overflow-x: auto;
    display: inline-block;
    padding: 10px;
}}
@font-face {{
    font-family: 'Unscii';
    src: url('https://viznut.fi/unscii/unscii-8.woff') format('woff');
    font-weight: normal;
    font-style: normal;
}}
</style>
</head>
<body><pre>"#, 
        bg = bg_color, fg = fg_color, font = font_family, fs = font_size, lh = line_height
    ));
    
    for row in 0..rows {
        for col in 0..cols {
            let idx = ascii_grid[(row * cols + col) as usize] as usize;
            let ch = CHAR_RAMP[idx];
            // Escape HTML
            match ch {
                '<' => html.push_str("<"),
                '>' => html.push_str(">"),
                '&' => html.push_str("&"),
                '"' => html.push_str("""),
                '\'' => html.push_str("'"),
                _ => html.push(ch),
            }
        }
        html.push('\n');
    }
    
    html.push_str("</pre></body></html>");
    html
}
```

### 6. ANSI Terminal (for CLI tools)
```rust
#[wasm_bindgen]
pub fn export_ansi(ascii_grid: &[u16], cols: u32, rows: u32, fg: u32, bg: u32) -> String {
    // fg/bg as packed RGB (0xRRGGBB)
    let fg_r = ((fg >> 16) & 0xFF) as u8;
    let fg_g = ((fg >> 8) & 0xFF) as u8;
    let fg_b = (fg & 0xFF) as u8;
    let bg_r = ((bg >> 16) & 0xFF) as u8;
    let bg_g = ((bg >> 8) & 0xFF) as u8;
    let bg_b = (bg & 0xFF) as u8;
    
    let mut out = String::new();
    out.push_str(&format!("\x1b[38;2;{};{};{}m\x1b[48;2;{};{};{}m", 
        fg_r, fg_g, fg_b, bg_r, bg_g, bg_b));
    
    for row in 0..rows {
        for col in 0..cols {
            let idx = ascii_grid[(row * cols + col) as usize] as usize;
            out.push(CHAR_RAMP[idx]);
        }
        out.push('\n');
    }
    
    out.push_str("\x1b[0m"); // Reset
    out
}
```

## Batch Export
```rust
#[wasm_bindgen]
pub struct BatchExportResult {
    pub png: Vec<u8>,
    pub webp: Vec<u8>,
    pub text: String,
    pub html: String,
}

#[wasm_bindgen]
pub fn export_all(
    ascii_grid: &[u16], 
    cols: u32, 
    rows: u32, 
    rgba: &[u8],
    fg: &str, bg: &str
) -> BatchExportResult {
    BatchExportResult {
        png: export_png(rgba, cols * 8, rows * 8),
        webp: export_webp(rgba, cols * 8, rows * 8, 85.0, false),
        text: export_text(ascii_grid, cols, rows),
        html: export_html(ascii_grid, cols, rows, fg, bg, "'Unscii', monospace", 1.1, 12),
    }
}
```

## Download Helper (JS)
```javascript
function downloadBlob(blob: Blob, filename: string) {
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}

// Usage
downloadBlob(new Blob([pngBytes], { type: 'image/png' }), 'art.png');
downloadBlob(new Blob([webpBytes], { type: 'image/webp' }), 'art.webp');
downloadBlob(new Blob([gifBytes], { type: 'image/gif' }), 'art.gif');
downloadBlob(new Blob([text], { type: 'text/plain' }), 'art.txt');
downloadBlob(new Blob([html], { type: 'text/html' }), 'art.html');
```

## Format Comparison
| Format | Transparency | Animation | Compression | Best For |
|--------|-------------|-----------|-------------|----------|
| PNG | ✅ | ❌ | Lossless | Archival, editing |
| WebP | ✅ | ✅ | Lossy/Lossless | Web, sharing |
| GIF | ⚠️ (1-bit) | ✅ | Lossy (256 colors) | Animation, compatibility |
| Text | ❌ | ❌ | None | Terminal, scripts |
| HTML | ✅ (CSS) | ❌ | None | Web embedding, sharing |
| ANSI | ✅ (24-bit) | ❌ | None | Terminal display |

## File Size Optimization
- **PNG**: Use `Compression::Best`, `FilterType::Paeth`
- **WebP**: Quality 85 (lossy) / 90 (lossless), method 4/6
- **GIF**: gifski-wasm quality 80, reduce fps for smaller files
- **HTML**: Minify CSS, use data URI for font if offline