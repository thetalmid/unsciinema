# WebGL Dithering Shaders

## Overview
Real-time dithering algorithms implemented in WebGL/WGSL for interactive artistic control. Based on Efecto's implementation and various open-source references.

## Dithering Algorithms

### 1. Floyd-Steinberg (Error Diffusion)
Classic 1976 algorithm. Sequential (hard to parallelize on GPU), but can run in WASM.
```rust
// CPU implementation in Rust/WASM
fn floyd_steinberg(pixels: &mut [u8], width: u32, height: u32, threshold: u8) {
    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            let old = pixels[idx];
            let new = if old > threshold { 255 } else { 0 };
            pixels[idx] = new;
            let err = old as i16 - new as i16;
            
            // Distribute error to neighbors
            if x + 1 < width { pixels[idx + 1] = saturate_add(pixels[idx + 1], err * 7 / 16); }
            if x > 0 && y + 1 < height { pixels[idx + width - 1] = saturate_add(pixels[idx + width - 1], err * 3 / 16); }
            if y + 1 < height { pixels[idx + width] = saturate_add(pixels[idx + width], err * 5 / 16); }
            if x + 1 < width && y + 1 < height { pixels[idx + width + 1] = saturate_add(pixels[idx + width + 1], err * 1 / 16); }
        }
    }
}
```

### 2. Atkinson (Macintosh 1984)
Distributes only 75% of error → higher contrast, "crunchy" look.
```rust
// Only 6 neighbors, divisor = 8 (so 6/8 = 75% distributed)
const ATKINSON_KERNEL: [(i32, i32, i32); 6] = [
    (1, 0, 1), (2, 0, 1), (-1, 1, 1), (0, 1, 1), (1, 1, 1), (0, 2, 1)
];
```

### 3. Ordered Dithering (Bayer) — **GPU-Friendly**
Fully parallelizable! Uses a threshold matrix.
```wgsl
// 4×4 Bayer matrix
const BAYER_4: array<f32, 16> = array(
    0.0,  8.0,  2.0, 10.0,
    12.0, 4.0, 14.0,  6.0,
    3.0, 11.0,  1.0,  9.0,
    15.0, 7.0, 13.0,  5.0
) / 16.0;

// 8×8 Bayer matrix (better quality)
const BAYER_8: array<f32, 64> = array( ... ) / 64.0;

// In fragment/compute shader:
let bayer_val = BAYER_4[(global_id.y & 3) * 4 + (global_id.x & 3)];
let threshold = bayer_val * dither_strength;
let binary = step(threshold, luma);  // 0.0 or 1.0
```

### 4. Blue Noise Dithering
Uses pre-computed blue noise texture for better visual quality.
```wgsl
@group(2) @binding(2) var blue_noise: texture_2d<f32>;
let noise = textureSampleLevel(blue_noise, sampler, uv * noise_scale, 0.0).r;
let binary = step(noise, luma);
```

### 5. Sierra / Jarvis-Judice-Ninke / Stucki / Burkes
Extended error diffusion kernels (more neighbors = smoother).
```rust
// Jarvis-Judice-Ninke (12 neighbors, 3 rows)
const JJN_KERNEL: [(i32, i32, i32); 12] = [
    (1, 0, 7), (2, 0, 5),
    (-2, 1, 3), (-1, 1, 5), (0, 1, 7), (1, 1, 5), (2, 1, 3),
    (-2, 2, 1), (-1, 2, 3), (0, 2, 5), (1, 2, 3), (2, 2, 1)
]; // divisor = 48
```

## GPU Implementation Strategy

### Option A: CPU (WASM) — Error Diffusion
- Floyd-Steinberg, Atkinson, JJN, Sierra are inherently sequential
- Run in Rust/WASM before uploading to GPU
- Good for quality, but slower for large images

### Option B: GPU (Compute Shader) — Ordered/Blue Noise
- Bayer, Blue Noise, Halftone are fully parallel
- Run in WebGL/WGSL compute shader
- Real-time at 60fps for any resolution

### Option C: Hybrid
- Ordered dithering on GPU for real-time preview
- Error diffusion on CPU for final export (higher quality)

## Efecto's Approach (Reference)
From https://efecto.app/ and Codrops article:
- **Dithering runs on CPU** (JavaScript) — error diffusion is sequential
- **Bloom runs on GPU** (WebGPU/WebGL)
- **ASCII runs on GPU** (WebGL shader) — independent cells
- WebGPU preferred; Canvas 2D fallback for Firefox

## Multi-Color Palette Dithering
```wgsl
// Palette: array of RGBA colors, ordered dark → light
const PALETTE: array<vec4<f32>, 4> = array(
    vec4(0.0, 0.0, 0.0, 1.0),   // Black
    vec4(0.5, 0.5, 0.5, 1.0),   // Dark gray
    vec4(0.75, 0.75, 0.75, 1.0), // Light gray
    vec4(1.0, 1.0, 1.0, 1.0)     // White
);

// Luminance → palette index
let lum = dot(color.rgb, vec3(0.299, 0.587, 0.114));
let index = floor(lum * f32(PALETTE.length - 1) + 0.5);
let dithered = PALETTE[u32(index)];
```

## Interactive Parameters (User Controls)
| Parameter | Range | Description |
|-----------|-------|-------------|
| `algorithm` | enum | none, floyd-steinberg, atkinson, bayer4, bayer8, blue-noise, sierra, jjn, stucki, burkes |
| `strength` | 0.0–1.0 | Blend between original and dithered |
| `threshold` | 0.0–1.0 | Global threshold (for simple thresholding) |
| `palette` | enum | 31 presets (terminal, gameboy, synthwave, etc.) + custom |
| `palette_colors` | 2–6 colors | Custom palette (hex strings) |
| `pixelation` | 1–10 | Block size for chunky look |
| `contrast` | 0.1–3.0 | Pre-dithering contrast adjustment |
| `gamma` | 0.5–2.5 | Gamma correction |

## Luminance Weights
```wgsl
// BT.709 (sRGB) - standard
const LUMA_BT709 = vec3(0.2126, 0.7152, 0.0722);

// BT.601 (legacy) - alternative
const LUMA_BT601 = vec3(0.299, 0.587, 0.114);

// Perceptual (CIE 1931)
const LUMA_CIE = vec3(0.2126, 0.7152, 0.0722);
```

## Color Palettes (from Efecto)
```typescript
const PRESETS = {
  // Monochrome
  terminal: ['#000000', '#333333', '#666666', '#999999', '#cccccc', '#ffffff'],
  gameboy: ['#0f380f', '#306230', '#8bac0f', '#9bbc0f'],
  
  // Warm
  sepia: ['#1a1a1a', '#4a3a2a', '#8a6a4a', '#d4b896', '#f5e6c8'],
  campfire: ['#0d0d0d', '#3d1a0d', '#8b3a1a', '#d47a3a', '#f5c896'],
  
  // Cool
  ocean: ['#000d1a', '#003d4d', '#007a8c', '#4db8d4', '#96e6f5'],
  ice: ['#00001a', '#002a6d', '#006ab8', '#4da6e6', '#96d4f5'],
  
  // Neon/Synthwave
  synthwave: ['#1a0033', '#4d0066', '#ff00ff', '#ff66ff', '#ffccff'],
  cyberpunk: ['#0d001a', '#3d0066', '#b800ff', '#ff00ff', '#ff80ff'],
  
  // Artistic
  ink: ['#000000', '#1a1a1a', '#333333', '#666666', '#cccccc'],
  gold: ['#0d0d00', '#3d3d00', '#b8b800', '#fff000', '#ffff80'],
};
```

## Implementation in Rust/WASM (for CPU dithering)
```rust
#[wasm_bindgen]
pub enum DitherAlgorithm {
    None = 0,
    FloydSteinberg = 1,
    Atkinson = 2,
    Bayer4 = 3,
    Bayer8 = 4,
    BlueNoise = 5,
    Sierra = 6,
    JarvisJudiceNinke = 7,
    Stucki = 8,
    Burkes = 9,
}

#[wasm_bindgen]
pub fn dither_image(
    pixels: &mut [u8],  // RGBA
    width: u32,
    height: u32,
    algorithm: DitherAlgorithm,
    strength: f32,
    palette: &[u32],    // Packed RGBA
) {
    match algorithm {
        DitherAlgorithm::Bayer4 | DitherAlgorithm::Bayer8 => {
            // Can be done in shader, but CPU fallback here
            ordered_dither(pixels, width, height, algorithm, strength, palette);
        }
        DitherAlgorithm::FloydSteinberg => {
            floyd_steinberg(pixels, width, height, strength, palette);
        }
        // ... etc
    }
}
```

## References
- **Efecto article**: https://tympanus.net/codrops/2026/01/04/efecto-building-real-time-ascii-and-dithering-effects-with-webgl-shaders/
- **Efecto live**: https://efecto.app/
- **Dithering Studio**: https://ditheringstudio.com/ (30+ algorithms)
- **DitherSpace**: https://github.com/landonjsmith/DitherSpace
- **dither-me**: https://github.com/maxtunelee/dither-me
- **halftone**: https://github.com/sefatlmn/halftone
- **The Book of Shaders**: https://thebookofshaders.com/