//! Image format loading using the `image` crate

use image::{ImageReader, DynamicImage, ImageFormat};
use std::io::Cursor;
use wasm_bindgen::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("Failed to decode image: {0}")]
    Decode(#[from] image::ImageError),
    #[error("Unsupported format")]
    UnsupportedFormat,
}

pub fn load_image(bytes: &[u8]) -> Result<DynamicImage, JsValue> {
    let format = detect_format(bytes).ok_or(JsValue::from_str("Unsupported image format"))?;
    let img = ImageReader::with_format(Cursor::new(bytes), format)
        .decode()
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(img)
}

fn detect_format(bytes: &[u8]) -> Option<ImageFormat> {
    if bytes.len() < 12 { return None; }
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") { return Some(ImageFormat::Png); }
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) { return Some(ImageFormat::Jpeg); }
    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") { return Some(ImageFormat::Gif); }
    if bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP" { return Some(ImageFormat::WebP); }
    if bytes.starts_with(b"BM") { return Some(ImageFormat::Bmp); }
    if bytes.starts_with(b"II\x2a\x00") || bytes.starts_with(b"MM\x00\x2a") { return Some(ImageFormat::Tiff); }
    None
}