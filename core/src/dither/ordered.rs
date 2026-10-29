//! Ordered dithering on the cell grid: Bayer 4x4/8x8, random noise.
//!
//! The salvaged threshold curve `(matrix + threshold) / 2` was nonstandard and
//! is replaced by the standard form shared with the rest of the pipeline:
//! add `strength * (n - 0.5) * step` to the threshold-adjusted luminance,
//! then multi-level quantize (see `dither::mod`).

use super::{step, LEVELS};

/// Standard 4x4 Bayer matrix (values k/16), from the salvage.
pub const BAYER_4: [f32; 16] = [
    0.0 / 16.0,
    8.0 / 16.0,
    2.0 / 16.0,
    10.0 / 16.0,
    12.0 / 16.0,
    4.0 / 16.0,
    14.0 / 16.0,
    6.0 / 16.0,
    3.0 / 16.0,
    11.0 / 16.0,
    1.0 / 16.0,
    9.0 / 16.0,
    15.0 / 16.0,
    7.0 / 16.0,
    13.0 / 16.0,
    5.0 / 16.0,
];

/// Standard 8x8 Bayer matrix (values k/64), from the salvage.
pub const BAYER_8: [f32; 64] = [
    0.0 / 64.0,
    32.0 / 64.0,
    8.0 / 64.0,
    40.0 / 64.0,
    2.0 / 64.0,
    34.0 / 64.0,
    10.0 / 64.0,
    42.0 / 64.0,
    48.0 / 64.0,
    16.0 / 64.0,
    56.0 / 64.0,
    24.0 / 64.0,
    50.0 / 64.0,
    18.0 / 64.0,
    58.0 / 64.0,
    26.0 / 64.0,
    12.0 / 64.0,
    44.0 / 64.0,
    4.0 / 64.0,
    36.0 / 64.0,
    14.0 / 64.0,
    46.0 / 64.0,
    6.0 / 64.0,
    38.0 / 64.0,
    60.0 / 64.0,
    28.0 / 64.0,
    52.0 / 64.0,
    20.0 / 64.0,
    62.0 / 64.0,
    30.0 / 64.0,
    54.0 / 64.0,
    22.0 / 64.0,
    3.0 / 64.0,
    35.0 / 64.0,
    11.0 / 64.0,
    43.0 / 64.0,
    1.0 / 64.0,
    33.0 / 64.0,
    9.0 / 64.0,
    41.0 / 64.0,
    51.0 / 64.0,
    19.0 / 64.0,
    59.0 / 64.0,
    27.0 / 64.0,
    49.0 / 64.0,
    17.0 / 64.0,
    57.0 / 64.0,
    25.0 / 64.0,
    15.0 / 64.0,
    47.0 / 64.0,
    7.0 / 64.0,
    39.0 / 64.0,
    13.0 / 64.0,
    45.0 / 64.0,
    5.0 / 64.0,
    37.0 / 64.0,
    63.0 / 64.0,
    31.0 / 64.0,
    55.0 / 64.0,
    23.0 / 64.0,
    61.0 / 64.0,
    29.0 / 64.0,
    53.0 / 64.0,
    21.0 / 64.0,
];

/// Position-hashed white noise in [0, 1) — "Random Noise" (algorithm 5).
/// Order-independent (pure function of cell coordinates), deterministic.
#[inline]
pub fn random_noise_at(x: u32, y: u32) -> f32 {
    let mut h = x
        .wrapping_mul(0x9E37_79B9)
        ^ y.wrapping_mul(0x85EB_CA77);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2545_F491);
    h ^= h >> 13;
    // Top 24 bits -> [0, 1)
    (h >> 8) as f32 / (1u32 << 24) as f32
}

#[inline]
fn ordered_level(l: f32, t_off: f32, n: f32, strength: f32) -> u16 {
    let l2 = l - t_off + strength * (n - 0.5) * step();
    let v = l2.clamp(0.0, 1.0);
    let lvl = (v / step()).round() as i32;
    lvl.clamp(0, (LEVELS - 1) as i32) as u16
}

/// Ordered dither with a static matrix (`size` x `size`).
pub fn ordered_dither(
    luma: &[f32],
    cols: u32,
    rows: u32,
    strength: f32,
    t_off: f32,
    matrix: &[f32],
    size: u32,
) -> Vec<u16> {
    let mut out = Vec::with_capacity(luma.len());
    for y in 0..rows {
        for x in 0..cols {
            let idx = (y * cols + x) as usize;
            let n = matrix[(y % size) as usize * size as usize + (x % size) as usize];
            out.push(ordered_level(luma[idx], t_off, n, strength));
        }
    }
    out
}

/// Ordered dither with a coordinate callback (noise sources).
pub fn ordered_dither_fn(
    luma: &[f32],
    cols: u32,
    rows: u32,
    strength: f32,
    t_off: f32,
    n_at: impl Fn(u32, u32) -> f32,
) -> Vec<u16> {
    let mut out = Vec::with_capacity(luma.len());
    for y in 0..rows {
        for x in 0..cols {
            let idx = (y * cols + x) as usize;
            let n = n_at(x, y);
            out.push(ordered_level(luma[idx], t_off, n, strength));
        }
    }
    out
}
