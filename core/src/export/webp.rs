//! WebP export using zenwebp (pure Rust)

use zenwebp::{EncodeRequest, LossyConfig, LosslessConfig, PixelLayout};
use wasm_bindgen::prelude::*;

pub fn export_webp(rgba: &[u8], width: u32, height: u32, quality: f32, lossless: bool) -> Vec<u8> {
    if lossless {
        EncodeRequest::lossless(
            &LosslessConfig::new()
                .with_quality(quality.clamp(0.0, 100.0))
                .with_method(6),
            rgba,
            PixelLayout::Rgba8,
            width,
            height,
        ).encode().unwrap_or_default()
    } else {
        EncodeRequest::lossy(
            &LossyConfig::new()
                .with_quality(quality.clamp(0.0, 100.0))
                .with_method(4),
            rgba,
            PixelLayout::Rgba8,
            width,
            height,
        ).encode().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webp_export_lossy() {
        let rgba = vec![255, 0, 0, 255].repeat(4); // 2x2 red
        let webp = export_webp(&rgba, 2, 2, 85.0, false);
        assert!(webp.len() > 0);
        // WebP signature: RIFF....WEBP
        assert!(webp.starts_with(b"RIFF"));
        assert_eq!(&webp[8..12], b"WEBP");
    }

    #[test]
    fn test_webp_export_lossless() {
        let rgba = vec![255, 0, 0, 255].repeat(4);
        let webp = export_webp(&rgba, 2, 2, 90.0, true);
        assert!(webp.len() > 0);
        assert!(webp.starts_with(b"RIFF"));
        assert_eq!(&webp[8..12], b"WEBP");
    }

    #[test]
    fn test_webp_export_1x1() {
        let rgba = vec![128, 128, 128, 255];
        let webp = export_webp(&rgba, 1, 1, 85.0, false);
        assert!(webp.len() > 0);
        assert!(webp.starts_with(b"RIFF"));
    }

    #[test]
    fn test_webp_quality_clamp() {
        let rgba = vec![255, 0, 0, 255].repeat(4);
        let webp1 = export_webp(&rgba, 2, 2, -10.0, false); // Clamped to 0
        let webp2 = export_webp(&rgba, 2, 2, 150.0, false); // Clamped to 100
        assert!(webp1.len() > 0);
        assert!(webp2.len() > 0);
    }

    #[test]
    fn test_webp_different_qualities() {
        let rgba = vec![255, 0, 0, 255].repeat(100); // 10x10
        let low = export_webp(&rgba, 10, 10, 10.0, false);
        let high = export_webp(&rgba, 10, 10, 95.0, false);
        // Higher quality should generally be larger (or equal)
        assert!(high.len() >= low.len());
    }
}