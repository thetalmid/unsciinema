//! Image preprocessing: contrast, gamma, resize

use image::{DynamicImage, GenericImageView, imageops::FilterType::CatmullRom};

/// Apply contrast adjustment (factor: 0.1-3.0, 1.0 = no change)
pub fn apply_contrast(rgba: &mut [u8], factor: f32) {
    if (factor - 1.0).abs() < f32::EPSILON { return; }
    let mid = 128.0;
    for pixel in rgba.chunks_exact_mut(4) {
        for c in &mut pixel[0..3] {
            let v = *c as f32;
            *c = (mid + (v - mid) * factor).clamp(0.0, 255.0) as u8;
        }
    }
}

/// Apply gamma correction (gamma: 0.5-2.5, 1.0 = no change)
pub fn apply_gamma(rgba: &mut [u8], gamma: f32) {
    if (gamma - 1.0).abs() < f32::EPSILON { return; }
    let inv_gamma = 1.0 / gamma;
    for pixel in rgba.chunks_exact_mut(4) {
        for c in &mut pixel[0..3] {
            let v = *c as f32 / 255.0;
            *c = (255.0 * v.powf(inv_gamma)).clamp(0.0, 255.0) as u8;
        }
    }
}

/// Resize image using Catmull-Rom filter (good for ASCII art)
pub fn resize_image(img: &DynamicImage, target_width: u32) -> DynamicImage {
    let (orig_w, orig_h) = img.dimensions();
    if target_width >= orig_w { return img.clone(); }
    let target_height = (orig_h as f32 * target_width as f32 / orig_w as f32).round() as u32;
    img.resize(target_width, target_height, CatmullRom)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::DynamicImage;

    #[test]
    fn test_contrast_noop() {
        let mut rgba = vec![100, 100, 100, 255];
        apply_contrast(&mut rgba, 1.0);
        assert_eq!(rgba[0], 100);
    }

    #[test]
    fn test_contrast_increase() {
        let mut rgba = vec![100, 100, 100, 255];
        apply_contrast(&mut rgba, 2.0);
        // 128 + (100-128)*2 = 128 - 56 = 72
        assert_eq!(rgba[0], 72);
    }

    #[test]
    fn test_contrast_decrease() {
        let mut rgba = vec![100, 100, 100, 255];
        apply_contrast(&mut rgba, 0.5);
        // 128 + (100-128)*0.5 = 128 - 14 = 114
        assert_eq!(rgba[0], 114);
    }

    #[test]
    fn test_contrast_clamp_high() {
        let mut rgba = vec![200, 200, 200, 255];
        apply_contrast(&mut rgba, 2.0);
        // 128 + (200-128)*2 = 128 + 144 = 272 -> clamped to 255
        assert_eq!(rgba[0], 255);
    }

    #[test]
    fn test_contrast_clamp_low() {
        let mut rgba = vec![50, 50, 50, 255];
        apply_contrast(&mut rgba, 2.0);
        // 128 + (50-128)*2 = 128 - 156 = -28 -> clamped to 0
        assert_eq!(rgba[0], 0);
    }

    #[test]
    fn test_contrast_only_rgb() {
        let mut rgba = vec![100, 150, 200, 128];
        apply_contrast(&mut rgba, 2.0);
        // Alpha should remain unchanged
        assert_eq!(rgba[3], 128);
    }

    #[test]
    fn test_gamma_noop() {
        let mut rgba = vec![128, 128, 128, 255];
        apply_gamma(&mut rgba, 1.0);
        assert_eq!(rgba[0], 128);
    }

    #[test]
    fn test_gamma_increase() {
        let mut rgba = vec![128, 128, 128, 255];
        apply_gamma(&mut rgba, 2.0);
        // (128/255)^0.5 * 255 ≈ 181
        assert!((179..=183).contains(&rgba[0]));
    }

    #[test]
    fn test_gamma_decrease() {
        let mut rgba = vec![128, 128, 128, 255];
        apply_gamma(&mut rgba, 0.5);
        // (128/255)^2 * 255 ≈ 64
        assert!((62..=66).contains(&rgba[0]));
    }

    #[test]
    fn test_gamma_clamp() {
        let mut rgba = vec![255, 255, 255, 255];
        apply_gamma(&mut rgba, 2.0);
        assert_eq!(rgba[0], 255);
    }

    #[test]
    fn test_gamma_only_rgb() {
        let mut rgba = vec![128, 128, 128, 100];
        apply_gamma(&mut rgba, 2.0);
        assert_eq!(rgba[3], 100);
    }
}