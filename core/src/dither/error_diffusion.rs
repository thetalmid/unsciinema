//! Error-diffusion dithering on the cell grid (multi-level, 16 ramp levels).
//!
//! Kernel weights are the salvaged ones (FS 7/3/5/1 over 16, Atkinson 6x 1/8,
//! Sierra /32, JJN /48, Stucki /42, Burkes /32) — but the quantizer is the
//! shared 16-level rule from `dither::mod`, so a gradient exercises the whole
//! ramp instead of collapsing to two glyphs.
//!
//! `strength` scales the diffused error, so `strength = 0` reduces exactly to
//! plain quantization (checked against `DitherAlgorithm::None` in tests).

use super::{step, DitherAlgorithm, LEVELS};

#[derive(Clone, Copy)]
struct Kernel {
    /// Neighbors as (dx, dy, weight) with dy > 0 or (dy == 0 && dx > 0).
    taps: &'static [(i32, i32, f32)],
    /// Sum of all weights (used for validation in tests).
    total: f32,
}

const FS: Kernel = Kernel {
    taps: &[(1, 0, 7.0), (-1, 1, 3.0), (0, 1, 5.0), (1, 1, 1.0)],
    total: 16.0,
};

const ATKINSON: Kernel = Kernel {
    taps: &[
        (1, 0, 1.0),
        (2, 0, 1.0),
        (-1, 1, 1.0),
        (0, 1, 1.0),
        (1, 1, 1.0),
        (0, 2, 1.0),
    ],
    total: 8.0, // 6/8 of the error is distributed (75%), classic Atkinson
};

const SIERRA: Kernel = Kernel {
    taps: &[
        (1, 0, 5.0),
        (2, 0, 3.0),
        (-2, 1, 2.0),
        (-1, 1, 4.0),
        (0, 1, 5.0),
        (1, 1, 4.0),
        (2, 1, 2.0),
        (-1, 2, 2.0),
        (0, 2, 3.0),
        (1, 2, 2.0),
    ],
    total: 32.0,
};

const JJN: Kernel = Kernel {
    taps: &[
        (1, 0, 7.0),
        (2, 0, 5.0),
        (-2, 1, 3.0),
        (-1, 1, 5.0),
        (0, 1, 7.0),
        (1, 1, 5.0),
        (2, 1, 3.0),
        (-2, 2, 1.0),
        (-1, 2, 3.0),
        (0, 2, 5.0),
        (1, 2, 3.0),
        (2, 2, 1.0),
    ],
    total: 48.0,
};

const STUCKI: Kernel = Kernel {
    taps: &[
        (1, 0, 8.0),
        (2, 0, 4.0),
        (-2, 1, 2.0),
        (-1, 1, 4.0),
        (0, 1, 8.0),
        (1, 1, 4.0),
        (2, 1, 2.0),
        (-2, 2, 1.0),
        (-1, 2, 2.0),
        (0, 2, 4.0),
        (1, 2, 2.0),
        (2, 2, 1.0),
    ],
    total: 42.0,
};

const BURKES: Kernel = Kernel {
    taps: &[
        (1, 0, 8.0),
        (2, 0, 4.0),
        (-2, 1, 2.0),
        (-1, 1, 4.0),
        (0, 1, 8.0),
        (1, 1, 4.0),
        (2, 1, 2.0),
    ],
    total: 32.0,
};

fn kernel_for(algo: DitherAlgorithm) -> Option<Kernel> {
    match algo {
        DitherAlgorithm::FloydSteinberg => Some(FS),
        DitherAlgorithm::Atkinson => Some(ATKINSON),
        DitherAlgorithm::Sierra => Some(SIERRA),
        DitherAlgorithm::JarvisJudiceNinke => Some(JJN),
        DitherAlgorithm::Stucki => Some(STUCKI),
        DitherAlgorithm::Burkes => Some(BURKES),
        _ => None,
    }
}

/// Run error diffusion over the cell grid.
pub fn diffuse(
    luma: &[f32],
    cols: u32,
    rows: u32,
    algo: DitherAlgorithm,
    strength: f32,
    t_off: f32,
) -> Vec<u16> {
    let kernel = match kernel_for(algo) {
        Some(k) => k,
        None => {
            // Not an error-diffusion algorithm: plain quantize.
            return luma.iter().map(|&l| super::plain_level(l, t_off)).collect();
        }
    };
    let w = cols as usize;
    let h = rows as usize;
    let mut work: Vec<f32> = luma.to_vec();
    let mut out = vec![0u16; luma.len()];
    let s = step();
    let inv_total = 1.0 / kernel.total;

    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;
            let v = work[idx];
            // Threshold-adjusted multi-level quantization.
            let qv = (v - t_off).clamp(0.0, 1.0);
            let lvl = ((qv / s).round() as i32).clamp(0, (LEVELS - 1) as i32) as u16;
            out[idx] = lvl;
            let quantized = lvl as f32 * s;
            let err = (v - quantized) * strength;
            if err == 0.0 {
                continue;
            }
            for &(dx, dy, weight) in kernel.taps {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx < 0 || ny < 0 || nx >= cols as i32 || ny >= rows as i32 {
                    continue;
                }
                let nidx = ny as usize * w + nx as usize;
                work[nidx] += err * weight * inv_total;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kernel_weights_are_normalized() {
        for algo in [
            DitherAlgorithm::FloydSteinberg,
            DitherAlgorithm::Atkinson,
            DitherAlgorithm::Sierra,
            DitherAlgorithm::JarvisJudiceNinke,
            DitherAlgorithm::Stucki,
            DitherAlgorithm::Burkes,
        ] {
            let k = kernel_for(algo).unwrap();
            let sum: f32 = k.taps.iter().map(|t| t.2).sum();
            let expected = if algo == DitherAlgorithm::Atkinson {
                6.0
            } else {
                k.total
            };
            assert!((sum - expected).abs() < 1e-6, "{algo:?}: {sum} != {expected}");
        }
    }
}
