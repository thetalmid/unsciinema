//! Dithering algorithms
//!
//! Error diffusion (CPU): Floyd-Steinberg, Atkinson, Sierra, JJN, Stucki, Burkes
//! Ordered (GPU-friendly): Bayer 4x4, Bayer 8x8, Blue Noise

pub mod error_diffusion;
pub mod ordered;
pub mod palette;

use crate::dither::error_diffusion::*;
use crate::dither::ordered::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

impl From<u32> for DitherAlgorithm {
    fn from(value: u32) -> Self {
        match value {
            0 => DitherAlgorithm::None,
            1 => DitherAlgorithm::FloydSteinberg,
            2 => DitherAlgorithm::Atkinson,
            3 => DitherAlgorithm::Bayer4,
            4 => DitherAlgorithm::Bayer8,
            5 => DitherAlgorithm::BlueNoise,
            6 => DitherAlgorithm::Sierra,
            7 => DitherAlgorithm::JarvisJudiceNinke,
            8 => DitherAlgorithm::Stucki,
            9 => DitherAlgorithm::Burkes,
            _ => DitherAlgorithm::None,
        }
    }
}

/// Apply dithering to luma buffer, producing a binary (0/255) pattern in `dithered_out`.
/// 
/// The `threshold` parameter (0.0-1.0) adjusts the quantization threshold.
/// The `strength` parameter (0.0-1.0) controls the dithering intensity.
/// 
/// For error diffusion: threshold shifts the quantization point
/// For ordered dithering: threshold is added to the matrix threshold
/// 
/// The input `luma` is NOT modified. The binary dithered result is written to `dithered_out`.
/// Both buffers must have the same length (width * height).
#[wasm_bindgen]
pub fn apply_dither(
    luma: &[u8],
    dithered_out: &mut [u8],
    width: u32,
    height: u32,
    algorithm: u32,
    strength: f32,
    threshold: f32,
) {
    let algorithm = DitherAlgorithm::from(algorithm);
    match algorithm {
        DitherAlgorithm::None => {
            // No dithering: just threshold the luma
            let thresh_val = (threshold * 255.0) as u8;
            for i in 0..luma.len() {
                dithered_out[i] = if luma[i] > thresh_val { 255 } else { 0 };
            }
        },
        DitherAlgorithm::FloydSteinberg => floyd_steinberg(luma, dithered_out, width, height, strength, threshold),
        DitherAlgorithm::Atkinson => atkinson(luma, dithered_out, width, height, strength, threshold),
        DitherAlgorithm::Sierra => sierra(luma, dithered_out, width, height, strength, threshold),
        DitherAlgorithm::JarvisJudiceNinke => jarvis_judice_ninke(luma, dithered_out, width, height, strength, threshold),
        DitherAlgorithm::Stucki => stucki(luma, dithered_out, width, height, strength, threshold),
        DitherAlgorithm::Burkes => burkes(luma, dithered_out, width, height, strength, threshold),
        DitherAlgorithm::Bayer4 => bayer_ordered(luma, dithered_out, width, height, strength, threshold, 4),
        DitherAlgorithm::Bayer8 => bayer_ordered(luma, dithered_out, width, height, strength, threshold, 8),
        DitherAlgorithm::BlueNoise => blue_noise(luma, dithered_out, width, height, strength, threshold),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_dither() {
        let luma = vec![100, 150, 200];
        let mut dithered = vec![0; 3];
        apply_dither(&luma, &mut dithered, 1, 3, 0, 1.0, 0.5);
        // With no dithering, just thresholds
        assert_eq!(dithered, vec![0, 255, 255]);
    }

    #[test]
    fn test_from_u32() {
        assert_eq!(DitherAlgorithm::from(0), DitherAlgorithm::None);
        assert_eq!(DitherAlgorithm::from(1), DitherAlgorithm::FloydSteinberg);
        assert_eq!(DitherAlgorithm::from(2), DitherAlgorithm::Atkinson);
        assert_eq!(DitherAlgorithm::from(3), DitherAlgorithm::Bayer4);
        assert_eq!(DitherAlgorithm::from(4), DitherAlgorithm::Bayer8);
        assert_eq!(DitherAlgorithm::from(5), DitherAlgorithm::BlueNoise);
        assert_eq!(DitherAlgorithm::from(6), DitherAlgorithm::Sierra);
        assert_eq!(DitherAlgorithm::from(7), DitherAlgorithm::JarvisJudiceNinke);
        assert_eq!(DitherAlgorithm::from(8), DitherAlgorithm::Stucki);
        assert_eq!(DitherAlgorithm::from(9), DitherAlgorithm::Burkes);
        assert_eq!(DitherAlgorithm::from(10), DitherAlgorithm::None);
    }
}