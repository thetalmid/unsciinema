//! Export formats

pub mod png;
pub mod webp;
pub mod text;
pub mod html;
pub mod ansi;
pub mod fonts;

use crate::export::png::export_png;
use crate::export::webp::export_webp;
use crate::export::text::export_text;
use crate::export::html::export_html;
use crate::export::ansi::export_ansi;
use crate::convert::grid::AsciiGrid;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn export_all(
    grid: &AsciiGrid,
    params: &ExportParams,
) -> Result<JsExportResult, JsValue> {
    let rgba = grid.rgba().ok_or_else(|| JsValue::from_str("No RGBA data"))?;
    let cols = grid.cols();
    let rows = grid.rows();
    let grid_data = grid.data();
    let width_px = grid.width_px();
    let height_px = grid.height_px();
    
    Ok(JsExportResult {
        png: export_png(&rgba, width_px, height_px)?,
        webp: export_webp(&rgba, width_px, height_px, params.webp_quality, params.webp_lossless)?,
        text: export_text(&grid_data, cols, rows),
        html: export_html(
            &grid_data, cols, rows,
            &params.html_fg, &params.html_bg, &params.html_font,
            params.html_line_height, params.html_font_size,
            match params.html_font_variant {
                0 => crate::font::variants::FontVariant::Unscii8,
                1 => crate::font::variants::FontVariant::Unscii8Alt,
                2 => crate::font::variants::FontVariant::Unscii8Fantasy,
                3 => crate::font::variants::FontVariant::Unscii8Mcr,
                4 => crate::font::variants::FontVariant::Unscii8Tall,
                5 => crate::font::variants::FontVariant::Unscii8Thin,
                6 => crate::font::variants::FontVariant::Unscii16,
                _ => crate::font::variants::FontVariant::Unscii8,
            }
        ),
        ansi: export_ansi(&grid_data, cols, rows, params.ansi_fg, params.ansi_bg),
    })
}

#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct ExportParams {
    pub webp_quality: f32,
    pub webp_lossless: bool,
    #[wasm_bindgen(getter_with_clone)]
    pub html_fg: String,
    #[wasm_bindgen(getter_with_clone)]
    pub html_bg: String,
    #[wasm_bindgen(getter_with_clone)]
    pub html_font: String,
    pub html_line_height: f32,
    pub html_font_size: u32,
    pub html_font_variant: u32,  // FontVariant discriminant
    pub ansi_fg: u32,
    pub ansi_bg: u32,
}

impl Default for ExportParams {
    fn default() -> Self {
        Self {
            webp_quality: 85.0,
            webp_lossless: false,
            html_fg: "#ffffff".to_string(),
            html_bg: "#000000".to_string(),
            html_font: "'Unscii', monospace".to_string(),
            html_line_height: 1.1,
            html_font_size: 12,
            html_font_variant: 0,  // FontVariant::Unscii8
            ansi_fg: 0xFFFFFFFF,
            ansi_bg: 0x000000FF,
        }
    }
}

// JsExportResult is the wasm-bindgen compatible version
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct JsExportResult {
    png: Vec<u8>,
    webp: Vec<u8>,
    text: String,
    html: String,
    ansi: String,
}

#[wasm_bindgen]
impl JsExportResult {
    #[wasm_bindgen(getter)]
    pub fn png(&self) -> Vec<u8> {
        self.png.clone()
    }
    
    #[wasm_bindgen(getter)]
    pub fn webp(&self) -> Vec<u8> {
        self.webp.clone()
    }
    
    #[wasm_bindgen(getter)]
    pub fn text(&self) -> String {
        self.text.clone()
    }
    
    #[wasm_bindgen(getter)]
    pub fn html(&self) -> String {
        self.html.clone()
    }
    
    #[wasm_bindgen(getter)]
    pub fn ansi(&self) -> String {
        self.ansi.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::convert::grid::AsciiGrid;

    #[test]
    fn test_export_params_default() {
        let params = ExportParams::default();
        assert_eq!(params.webp_quality, 85.0);
        assert!(!params.webp_lossless);
        assert_eq!(params.html_fg, "#ffffff");
        assert_eq!(params.html_bg, "#000000");
        assert_eq!(params.html_line_height, 1.1);
        assert_eq!(params.html_font_size, 12);
        assert_eq!(params.ansi_fg, 0xFFFFFFFF);
        assert_eq!(params.ansi_bg, 0x000000FF);
    }

    #[test]
    fn test_export_all_creates_all_formats() {
        let mut grid = AsciiGrid::new(2, 2);
        // 2x2 grid = 4 cells, each 8x8 = 64 pixels = 16*16 = 256 total pixels
        // 256 pixels * 4 channels = 1024 bytes
        grid.set_rgba(vec![255u8; 1024]); // 16x16 white
        
        let params = ExportParams::default();
        let result = export_all(&grid, &params).unwrap();
        
        assert!(!result.png.is_empty());
        assert!(!result.webp.is_empty());
        assert!(!result.text.is_empty());
        assert!(!result.html.is_empty());
        assert!(!result.ansi.is_empty());
    }

    #[test]
    fn test_export_result_png_valid() {
        let mut grid = AsciiGrid::new(1, 1);
        // 1x1 grid = 1 cell = 8x8 = 64 pixels = 256 bytes
        grid.set_rgba(vec![255, 0, 0, 255].repeat(64));
        
        let params = ExportParams::default();
        let result = export_all(&grid, &params).unwrap();
        
        assert_eq!(&result.png[0..8], &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
    }

    #[test]
    fn test_export_result_webp_valid() {
        let mut grid = AsciiGrid::new(1, 1);
        grid.set_rgba(vec![255, 0, 0, 255].repeat(64));
        
        let params = ExportParams::default();
        let result = export_all(&grid, &params).unwrap();
        
        assert!(result.webp.starts_with(b"RIFF"));
        assert_eq!(&result.webp[8..12], b"WEBP");
    }
}