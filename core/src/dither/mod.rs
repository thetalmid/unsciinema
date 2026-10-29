//! Dithering dispatch for the glyph pipeline.
//!
//! Dithering runs on the **cell-luminance grid** (not the full-res image) so
//! the pattern survives cell averaging — ordered/error-diffusion kernels
//! quantize cell luminance directly into the 16-level glyph ramp.
//!
//! ## Shared mathematics (implemented once, here)
//!
//! With `N = 16` ramp levels and `step = 1/(N-1)`:
//!
//! - threshold offset: `t_off = (threshold - 0.5) * step`
//!   (`threshold = 0.5` — the default — is exactly neutral)
//! - plain quantization (`None` algorithm and `strength = 0` limit):
//!   `level = round(clamp(l - t_off, 0, 1) / step)`
//! - ordered/noise dithering:
//!   `l' = l - t_off + strength * (n - 0.5) * step`, then plain-quantize `l'`
//!   (`strength = 0` reproduces `None` bit-for-bit; `n ∈ [0,1)` is the matrix
//!   or noise value; amplitude is half a quantization step at full strength)
//! - error diffusion: quantize `work[i]` with the threshold-adjusted rule,
//!   then diffuse `err = (work[i] - level*step) * strength` with the kernel
//!   weights (Atkinson distributes 6/8 of the error, like the salvage).
//!
//! Algorithm IDs follow the salvaged mapping 0..=9; ID 5 stays **Random
//! Noise** (position-hashed white noise, NOT blue noise). ID 10 is genuine
//! tileable **blue noise** (void-and-cluster-style best-candidate mask,
//! generated deterministically at first use).

pub mod blue_noise;
pub mod error_diffusion;
pub mod ordered;
pub mod palette;

use crate::error::{CoreError, Result};
use crate::limits::check_f32;

/// Number of ramp levels the quantizer maps into.
pub const LEVELS: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DitherAlgorithm {
    None = 0,
    FloydSteinberg = 1,
    Atkinson = 2,
    Bayer4 = 3,
    Bayer8 = 4,
    RandomNoise = 5,
    Sierra = 6,
    JarvisJudiceNinke = 7,
    Stucki = 8,
    Burkes = 9,
    BlueNoise = 10,
}

impl DitherAlgorithm {
    pub fn from_u32(v: u32) -> Option<DitherAlgorithm> {
        Some(match v {
            0 => DitherAlgorithm::None,
            1 => DitherAlgorithm::FloydSteinberg,
            2 => DitherAlgorithm::Atkinson,
            3 => DitherAlgorithm::Bayer4,
            4 => DitherAlgorithm::Bayer8,
            5 => DitherAlgorithm::RandomNoise,
            6 => DitherAlgorithm::Sierra,
            7 => DitherAlgorithm::JarvisJudiceNinke,
            8 => DitherAlgorithm::Stucki,
            9 => DitherAlgorithm::Burkes,
            10 => DitherAlgorithm::BlueNoise,
            _ => return None,
        })
    }

    pub fn is_error_diffusion(self) -> bool {
        matches!(
            self,
            DitherAlgorithm::FloydSteinberg
                | DitherAlgorithm::Atkinson
                | DitherAlgorithm::Sierra
                | DitherAlgorithm::JarvisJudiceNinke
                | DitherAlgorithm::Stucki
                | DitherAlgorithm::Burkes
        )
    }
}

pub fn validate_strength(v: f32) -> Result<f32> {
    check_f32("dither_strength", v, 0.0, 1.0)
}

pub fn validate_threshold(v: f32) -> Result<f32> {
    check_f32("threshold", v, 0.0, 1.0)
}

#[inline]
pub fn step() -> f32 {
    1.0 / (LEVELS as f32 - 1.0)
}

/// Threshold offset in luminance units.
#[inline]
pub fn threshold_offset(threshold: f32) -> f32 {
    (threshold - 0.5) * step()
}

/// Plain (undithered) multi-level quantization of a luminance in [0,1].
#[inline]
pub fn plain_level(l: f32, t_off: f32) -> u16 {
    let v = (l - t_off).clamp(0.0, 1.0);
    let lvl = (v / step()).round() as i32;
    lvl.clamp(0, (LEVELS - 1) as i32) as u16
}

/// Quantize a cell-luminance grid (values 0..1, row-major cols*rows) into
/// glyph ramp levels using the selected algorithm.
pub fn quantize_grid(
    luma: &[f32],
    cols: u32,
    rows: u32,
    algo: DitherAlgorithm,
    strength: f32,
    threshold: f32,
) -> Result<Vec<u16>> {
    let cells = crate::limits::checked_cells(cols, rows)?;
    if luma.len() != cells {
        return Err(CoreError::invalid(format!(
            "luma grid has {} values, expected {cells}",
            luma.len()
        )));
    }
    let strength = validate_strength(strength)?;
    let threshold = validate_threshold(threshold)?;
    let t_off = threshold_offset(threshold);

    if algo == DitherAlgorithm::None || strength == 0.0 {
        return Ok(luma.iter().map(|&l| plain_level(l, t_off)).collect());
    }

    match algo {
        DitherAlgorithm::None => unreachable!(),
        DitherAlgorithm::Bayer4 => {
            Ok(ordered::ordered_dither(luma, cols, rows, strength, t_off, &ordered::BAYER_4, 4))
        }
        DitherAlgorithm::Bayer8 => {
            Ok(ordered::ordered_dither(luma, cols, rows, strength, t_off, &ordered::BAYER_8, 8))
        }
        DitherAlgorithm::RandomNoise => Ok(ordered::ordered_dither_fn(
            luma,
            cols,
            rows,
            strength,
            t_off,
            ordered::random_noise_at,
        )),
        DitherAlgorithm::BlueNoise => {
            let mask = blue_noise::blue_noise_mask();
            Ok(ordered::ordered_dither_fn(luma, cols, rows, strength, t_off, |x, y| {
                mask[((y % 32) * 32 + (x % 32)) as usize]
            }))
        }
        a if a.is_error_diffusion() => Ok(error_diffusion::diffuse(
            luma,
            cols,
            rows,
            a,
            strength,
            t_off,
        )),
        _ => Err(CoreError::invalid("unknown dither algorithm")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neutral_threshold_and_strength_match_none() {
        let grid: Vec<f32> = (0..64).map(|i| (i as f32) / 63.0).collect();
        let none = quantize_grid(&grid, 8, 8, DitherAlgorithm::None, 1.0, 0.5).unwrap();
        // strength 0 must equal None for every algorithm.
        for algo in [
            DitherAlgorithm::FloydSteinberg,
            DitherAlgorithm::Atkinson,
            DitherAlgorithm::Bayer4,
            DitherAlgorithm::Bayer8,
            DitherAlgorithm::RandomNoise,
            DitherAlgorithm::Sierra,
            DitherAlgorithm::JarvisJudiceNinke,
            DitherAlgorithm::Stucki,
            DitherAlgorithm::Burkes,
            DitherAlgorithm::BlueNoise,
        ] {
            let z = quantize_grid(&grid, 8, 8, algo, 0.0, 0.5).unwrap();
            assert_eq!(z, none, "strength 0 mismatch for {algo:?}");
        }
    }

    #[test]
    fn plain_quantization_endpoints_and_mid() {
        assert_eq!(plain_level(0.0, 0.0), 0);
        assert_eq!(plain_level(1.0, 0.0), 15);
        // 128/255 ~ 0.502 -> round(7.53) = 8 (same as the salvaged mapper).
        assert_eq!(plain_level(128.0 / 255.0, 0.0), 8);
        // Threshold shifts by half a step per 0.5 of threshold.
        assert_eq!(plain_level(0.5, threshold_offset(1.0)), 7);
    }

    #[test]
    fn error_diffusion_keeps_all_levels_on_gradient() {
        let w = 64;
        let grid: Vec<f32> = (0..w * 8)
            .map(|i| i as f32 / (w * 8 - 1) as f32)
            .collect();
        let out = quantize_grid(&grid, w, 8, DitherAlgorithm::FloydSteinberg, 1.0, 0.5).unwrap();
        let mut seen = [false; 16];
        for &v in &out {
            seen[v as usize] = true;
        }
        // A 16-level ramp must not collapse to two glyphs.
        let count = seen.iter().filter(|&&s| s).count();
        assert!(count >= 10, "only {count} distinct levels");
    }

    #[test]
    fn random_noise_is_deterministic() {
        let grid = vec![0.5f32; 100];
        let a = quantize_grid(&grid, 10, 10, DitherAlgorithm::RandomNoise, 1.0, 0.5).unwrap();
        let b = quantize_grid(&grid, 10, 10, DitherAlgorithm::RandomNoise, 1.0, 0.5).unwrap();
        assert_eq!(a, b);
        // ...and actually varies spatially.
        assert!(a.iter().any(|&v| v != a[0]) || a.windows(2).any(|w| w[0] != w[1]));
    }
}
