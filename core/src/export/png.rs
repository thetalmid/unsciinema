//! PNG export

use png::{Encoder, ColorType, BitDepth, Compression};
use wasm_bindgen::prelude::*;

pub fn export_png(rgba: &[u8], width: u32, height: u32) -> Result<Vec<u8>, JsValue> {
    let mut buf = Vec::new();
    {
        let mut encoder = Encoder::new(&mut buf, width, height);
        encoder.set_color(ColorType::Rgba);
        encoder.set_depth(BitDepth::Eight);
        encoder.set_compression(Compression::default());
        let mut writer = encoder.write_header()
            .map_err(|e| JsValue::from_str(&format!("PNG header error: {}", e)))?;
        writer.write_image_data(rgba)
            .map_err(|e| JsValue::from_str(&format!("PNG data error: {}", e)))?;
    }
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_png_export_2x2() {
        let rgba = vec![
            255, 0, 0, 255,   // Red
            0, 255, 0, 255,   // Green
            0, 0, 255, 255,   // Blue
            255, 255, 255, 255, // White
        ];
        let png = export_png(&rgba, 2, 2).unwrap();
        assert!(png.len() > 0);
        // PNG signature
        assert_eq!(&png[0..8], &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
    }

    #[test]
    fn test_png_export_1x1() {
        let rgba = vec![128, 128, 128, 255];
        let png = export_png(&rgba, 1, 1).unwrap();
        assert!(png.len() > 0);
        assert_eq!(&png[0..8], &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
    }

    #[test]
    fn test_png_export_larger() {
        let rgba = vec![255, 0, 0, 255].repeat(100); // 10x10 red
        let png = export_png(&rgba, 10, 10).unwrap();
        assert!(png.len() > 0);
        assert_eq!(&png[0..8], &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
    }
}