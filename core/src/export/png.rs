//! PNG export (salvaged `png` crate fragment, now with full validation).

use crate::error::{CoreError, Result};
use crate::limits::checked_rgba_len;

pub fn export_png(rgba: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
    let expected = checked_rgba_len(width, height, 4)?;
    if rgba.len() != expected {
        return Err(CoreError::export(format!(
            "PNG buffer has {} bytes, expected {expected} for {width}x{height} RGBA8",
            rgba.len()
        )));
    }
    let mut buf = Vec::with_capacity(expected / 4);
    {
        let mut encoder = png::Encoder::new(&mut buf, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder
            .write_header()
            .map_err(|e| CoreError::export(format!("PNG header: {e}")))?;
        writer
            .write_image_data(rgba)
            .map_err(|e| CoreError::export(format!("PNG data: {e}")))?;
    }
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn png_roundtrip_pixels() {
        let w = 5u32;
        let h = 3u32;
        let mut rgba = Vec::new();
        for i in 0..(w * h) {
            rgba.extend_from_slice(&[(i * 7) as u8, (i * 13) as u8, (i * 29) as u8, 200]);
        }
        let bytes = export_png(&rgba, w, h).unwrap();
        assert_eq!(&bytes[0..8], &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]);
        let decoded = image::load_from_memory(&bytes).unwrap().into_rgba8();
        assert_eq!(decoded.dimensions(), (w, h));
        assert_eq!(decoded.as_raw(), &rgba);
    }

    #[test]
    fn png_rejects_bad_lengths() {
        assert!(export_png(&[0u8; 10], 2, 2).is_err());
        assert!(export_png(&[], 0, 5).is_err());
    }
}
