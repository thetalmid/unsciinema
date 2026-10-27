//! Image format loading using the `image` crate

use gamut_exif::{Exif, ExifTag};
use image::{DynamicImage, ImageFormat, ImageReader};
use std::io::Cursor;
use wasm_bindgen::prelude::*;

pub fn load_image(bytes: &[u8]) -> Result<DynamicImage, JsValue> {
    let format = detect_format(bytes).ok_or(JsValue::from_str("Unsupported image format"))?;
    let mut img = ImageReader::with_format(Cursor::new(bytes), format)
        .decode()
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Apply EXIF orientation if present (mainly for JPEG/TIFF)
    if let Some(orientation) = read_exif_orientation(bytes) {
        img = apply_orientation(img, orientation);
    }

    Ok(img)
}

fn detect_format(bytes: &[u8]) -> Option<ImageFormat> {
    if bytes.len() < 12 {
        return None;
    }
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Some(ImageFormat::Png);
    }
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Some(ImageFormat::Jpeg);
    }
    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        return Some(ImageFormat::Gif);
    }
    if bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP" {
        return Some(ImageFormat::WebP);
    }
    if bytes.starts_with(b"BM") {
        return Some(ImageFormat::Bmp);
    }
    if bytes.starts_with(b"II\x2a\x00") || bytes.starts_with(b"MM\x00\x2a") {
        return Some(ImageFormat::Tiff);
    }
    None
}

/// Read EXIF orientation from image bytes using gamut-exif
fn read_exif_orientation(bytes: &[u8]) -> Option<u32> {
    // gamut-exif can parse EXIF from JPEG and TIFF
    let exif = Exif::parse(bytes).ok()?;

    // Get Orientation tag (0x0112) from 0th IFD
    let orientation_value = exif.get_tag(ExifTag::Orientation)?;
    // Extract u16 value from the EXIF value
    match orientation_value {
        gamut_exif::Value::Short(vec) => vec.first().copied().map(|v| v as u32),
        gamut_exif::Value::Long(vec) => vec.first().copied(),
        _ => None,
    }
}

/// Apply EXIF orientation transformation
fn apply_orientation(img: DynamicImage, orientation: u32) -> DynamicImage {
    match orientation {
        1 => img,                    // Normal
        2 => img.fliph(),            // Mirror horizontal
        3 => img.rotate180(),        // Rotate 180
        4 => img.flipv(),            // Mirror vertical
        5 => img.fliph().rotate90(), // Mirror horizontal + rotate 90 CW
        6 => img.rotate90(),         // Rotate 90 CW
        7 => img.fliph().rotate270(), // Mirror horizontal + rotate 90 CCW
        8 => img.rotate270(),        // Rotate 90 CCW
        _ => img,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_format_png() {
        let png_header = b"\x89PNG\r\n\x1a\n\x00\x00\x00\x0D";
        assert_eq!(detect_format(png_header), Some(ImageFormat::Png));
    }

    #[test]
    fn test_detect_format_jpeg() {
        let jpeg_header = &[0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01];
        assert_eq!(detect_format(jpeg_header), Some(ImageFormat::Jpeg));
    }

    #[test]
    fn test_detect_format_gif87a() {
        let gif_header = b"GIF87a\x00\x00\x00\x00\x00\x00";
        assert_eq!(detect_format(gif_header), Some(ImageFormat::Gif));
    }

    #[test]
    fn test_detect_format_gif89a() {
        let gif_header = b"GIF89a\x00\x00\x00\x00\x00\x00";
        assert_eq!(detect_format(gif_header), Some(ImageFormat::Gif));
    }

    #[test]
    fn test_detect_format_webp() {
        let webp_header = b"RIFF\x00\x00\x00\x00WEBP";
        assert_eq!(detect_format(webp_header), Some(ImageFormat::WebP));
    }

    #[test]
    fn test_detect_format_bmp() {
        let bmp_header = b"BM\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
        assert_eq!(detect_format(bmp_header), Some(ImageFormat::Bmp));
    }

    #[test]
    fn test_detect_format_tiff_le() {
        let tiff_header = b"II\x2a\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
        assert_eq!(detect_format(tiff_header), Some(ImageFormat::Tiff));
    }

    #[test]
    fn test_detect_format_tiff_be() {
        let tiff_header = b"MM\x00\x2a\x00\x00\x00\x00\x00\x00\x00\x00\x00";
        assert_eq!(detect_format(tiff_header), Some(ImageFormat::Tiff));
    }

    #[test]
    fn test_detect_format_unknown() {
        let unknown = b"UNKNOWN";
        assert_eq!(detect_format(unknown), None);
    }

    #[test]
    fn test_apply_orientation_noop() {
        let img = DynamicImage::new_rgb8(10, 10);
        let result = apply_orientation(img.clone(), 1);
        assert_eq!(result.width(), img.width());
        assert_eq!(result.height(), img.height());
    }

    #[test]
    fn test_apply_orientation_rotate180() {
        let img = DynamicImage::new_rgb8(20, 10);
        let result = apply_orientation(img, 3);
        assert_eq!(result.width(), 20);
        assert_eq!(result.height(), 10);
    }

    #[test]
    fn test_apply_orientation_rotate90() {
        let img = DynamicImage::new_rgb8(20, 10);
        let result = apply_orientation(img, 6);
        assert_eq!(result.width(), 10);
        assert_eq!(result.height(), 20);
    }

    #[test]
    fn test_apply_orientation_rotate270() {
        let img = DynamicImage::new_rgb8(20, 10);
        let result = apply_orientation(img, 8);
        assert_eq!(result.width(), 10);
        assert_eq!(result.height(), 20);
    }
}