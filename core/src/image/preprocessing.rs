//! Preprocessing: aspect-preserving resize, contrast, gamma.
//!
//! Pipeline order (documented in INTEGRATION.md):
//!   decode -> EXIF orientation -> resize (optional) -> contrast -> gamma -> alpha stage
//!
//! Contrast and gamma operate on *encoded* sRGB channels (non-linear), leave
//! the alpha channel untouched, and are exact no-ops at their neutral value
//! `1.0`.

use crate::error::{CoreError, Result};
use crate::limits::check_f32;
use image::{imageops, RgbaImage};

/// Contrast factor range (neutral 1.0, from the salvaged function).
pub fn validate_contrast(v: f32) -> Result<f32> {
    check_f32("contrast", v, 0.1, 3.0)
}

/// Gamma range (neutral 1.0, from the salvaged function: `pow(c, 1/gamma)`,
/// so gamma > 1 brightens, gamma < 1 darkens).
pub fn validate_gamma(v: f32) -> Result<f32> {
    check_f32("gamma", v, 0.5, 2.5)
}

/// Salvaged contrast, kept: `mid + (v - mid) * factor`, clamped, RGB only.
/// Defaults 1.0/1.0 must be neutral -> early return at exactly 1.0.
pub fn apply_contrast(rgba: &mut [u8], factor: f32) {
    if factor == 1.0 {
        return;
    }
    let mid = 128.0f32;
    for pixel in rgba.chunks_exact_mut(4) {
        for c in &mut pixel[0..3] {
            let v = *c as f32;
            *c = (mid + (v - mid) * factor).clamp(0.0, 255.0) as u8;
        }
    }
}

/// Salvaged gamma, kept: `255 * pow(v/255, 1/gamma)`, clamped, RGB only.
pub fn apply_gamma(rgba: &mut [u8], gamma: f32) {
    if gamma == 1.0 {
        return;
    }
    let inv_gamma = 1.0 / gamma;
    for pixel in rgba.chunks_exact_mut(4) {
        for c in &mut pixel[0..3] {
            let v = *c as f32 / 255.0;
            *c = (255.0 * v.powf(inv_gamma)).clamp(0.0, 255.0) as u8;
        }
    }
}

/// Resize to exactly `target_width` source pixels, keeping aspect ratio
/// (`target_width == 0` means no forced resize). Transparent edges are
/// protected against halos by resizing premultiplied RGBA and un-premultiplying
/// afterwards. Returns `None` when no resize is needed.
pub fn resize_target(
    img: &RgbaImage,
    target_width: u32,
) -> Result<Option<RgbaImage>> {
    let (w, h) = img.dimensions();
    if target_width == 0 {
        return Ok(None);
    }
    if target_width > crate::limits::MAX_SOURCE_DIM {
        return Err(CoreError::limit(format!(
            "target width {target_width} exceeds max {}",
            crate::limits::MAX_SOURCE_DIM
        )));
    }
    if target_width == w {
        return Ok(None);
    }
    let target_h = (((h as f64) * (target_width as f64) / (w as f64)).round() as u32).max(1);

    // Premultiply -> resize -> un-premultiply.
    let mut prem = img.clone();
    for px in prem.chunks_exact_mut(4) {
        let a = px[3] as u32;
        px[0] = ((px[0] as u32 * a) / 255) as u8;
        px[1] = ((px[1] as u32 * a) / 255) as u8;
        px[2] = ((px[2] as u32 * a) / 255) as u8;
    }
    let resized = imageops::resize(
        &prem,
        target_width,
        target_h,
        imageops::FilterType::CatmullRom,
    );
    let mut out = resized;
    for px in out.chunks_exact_mut(4) {
        let a = px[3] as u32;
        if a == 0 {
            px[0] = 0;
            px[1] = 0;
            px[2] = 0;
        } else {
            px[0] = (((px[0] as u32) * 255) / a).min(255) as u8;
            px[1] = (((px[1] as u32) * 255) / a).min(255) as u8;
            px[2] = (((px[2] as u32) * 255) / a).min(255) as u8;
        }
    }
    Ok(Some(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contrast_neutral_is_identity() {
        let mut a = vec![0u8, 10, 128, 200, 255, 4, 64, 9];
        let orig = a.clone();
        apply_contrast(&mut a, 1.0);
        assert_eq!(a, orig);
    }

    #[test]
    fn contrast_pushes_around_mid() {
        let mut a = vec![200u8, 0, 0, 255];
        apply_contrast(&mut a, 2.0);
        // 128 + (200-128)*2 = 272 -> 255
        assert_eq!(a[0], 255);
        // alpha untouched
        assert_eq!(a[3], 255);
        let mut b = vec![64u8, 0, 0, 255];
        apply_contrast(&mut b, 0.5);
        assert_eq!(b[0], (128.0 + (64.0 - 128.0) * 0.5) as u8);
    }

    #[test]
    fn gamma_direction_is_documented() {
        let mut a = vec![128u8, 0, 0, 255];
        apply_gamma(&mut a, 2.0);
        // pow(128/255, 0.5)*255 ~ 181 -> brighter
        assert!(a[0] > 128 && a[0] < 190, "got {}", a[0]);
        let mut b = vec![128u8, 0, 0, 255];
        apply_gamma(&mut b, 0.5);
        // pow(128/255, 2)*255 ~ 64.5 -> darker
        assert!(b[0] < 128 && b[0] > 50, "got {}", b[0]);
        let mut c = vec![128u8, 1, 2, 255];
        apply_gamma(&mut c, 1.0);
        assert_eq!(c, vec![128, 1, 2, 255]);
    }

    #[test]
    fn resize_exact_width_and_aspect() {
        let img = RgbaImage::from_pixel(10, 4, image::Rgba([1, 2, 3, 255]));
        let out = resize_target(&img, 20).unwrap().unwrap();
        assert_eq!(out.dimensions(), (20, 8));
        let out0 = resize_target(&img, 0).unwrap();
        assert!(out0.is_none());
        let out_same = resize_target(&img, 10).unwrap();
        assert!(out_same.is_none());
        // Tiny height rounds to at least 1.
        let thin = RgbaImage::from_pixel(100, 1, image::Rgba([9, 9, 9, 255]));
        let o = resize_target(&thin, 10).unwrap().unwrap();
        assert_eq!(o.dimensions(), (10, 1));
    }
}
