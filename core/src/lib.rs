//! Unscii Image Converter Core - Rust/WASM
//!
//! Core image-to-ASCII conversion logic compiled to WebAssembly.

pub mod font;
pub mod image;
pub mod dither;
pub mod convert;
pub mod export;

use font::{load_font_variant, FontAtlas};
use font::variants::FontVariant;
use image::ImageProcessor;
use convert::{convert_image, AsciiGrid, ConvertParams};
use export::{export_all, ExportParams, JsExportResult};
use wasm_bindgen::prelude::*;

// Panic hook for better error messages in browser
#[cfg(feature = "console_error_panic_hook")]
fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    init_panic_hook();
}

/// Main converter struct exposed to JavaScript
#[wasm_bindgen]
pub struct UnsciiConverter {
    font_atlas: FontAtlas,
    font_variant: FontVariant,
    image_processor: Option<image::ImageProcessor>,
    max_file_size: usize,
}

#[wasm_bindgen]
impl UnsciiConverter {
    /// Create a new converter with the specified font variant
    #[wasm_bindgen(constructor)]
    pub fn new(variant: u32) -> Result<UnsciiConverter, JsValue> {
        let font_variant = match variant {
            0 => FontVariant::Unscii8,
            1 => FontVariant::Unscii8Alt,
            2 => FontVariant::Unscii8Fantasy,
            3 => FontVariant::Unscii8Mcr,
            4 => FontVariant::Unscii8Tall,
            5 => FontVariant::Unscii8Thin,
            6 => FontVariant::Unscii16,
            _ => FontVariant::Unscii8,
        };
        
        let font_atlas = load_font_variant(font_variant)?;
        Ok(UnsciiConverter { 
            font_atlas, 
            font_variant, 
            image_processor: None,
            max_file_size: 50 * 1024 * 1024, // 50 MB default
        })
    }

    /// Get font atlas as RGBA bytes for GPU upload (128×8 = 4096 bytes)
    #[wasm_bindgen]
    pub fn font_atlas(&self) -> Vec<u8> {
        self.font_atlas.rgba_bytes().to_vec()
    }

    /// Get font atlas dimensions [width, height]
    #[wasm_bindgen]
    pub fn font_atlas_size(&self) -> Vec<u32> {
        self.font_atlas.dimensions_array().to_vec()
    }

    /// Get character ramp (16 glyphs, density-ordered)
    #[wasm_bindgen]
    pub fn char_ramp(&self) -> Vec<u16> {
        self.font_atlas.char_ramp.iter().map(|&c| c as u16).collect()
    }

    /// Get font variant index
    #[wasm_bindgen]
    pub fn font_variant(&self) -> u32 {
        self.font_variant as u32
    }

    /// Load and decode image once, store for repeated conversions
    #[wasm_bindgen]
    pub fn load_image(&mut self, image_bytes: &[u8]) -> Result<(), JsValue> {
        if image_bytes.len() > self.max_file_size {
            return Err(JsValue::from_str(&format!(
                "File too large: {} bytes (max: {} bytes)",
                image_bytes.len(), self.max_file_size
            )));
        }
        let processor = ImageProcessor::new(image_bytes)?;
        self.image_processor = Some(processor);
        Ok(())
    }

    /// Convert using already-loaded image (decoding skipped)
    #[wasm_bindgen]
    pub fn convert_loaded(&mut self, params: &ConvertParams) -> Result<AsciiGrid, JsValue> {
        let processor = self.image_processor.as_mut()
            .ok_or_else(|| JsValue::from_str("No image loaded. Call load_image() first."))?;
        convert_image(processor, params, &self.font_atlas)
    }

    /// Convert image to ASCII grid with current parameters (legacy: decodes each time)
    /// Returns AsciiGrid with glyph indices and rendered RGBA
    #[wasm_bindgen]
    pub fn convert(
        &self,
        image_bytes: &[u8],
        params: &ConvertParams,
    ) -> Result<AsciiGrid, JsValue> {
        let mut processor = ImageProcessor::new(image_bytes)?;
        convert_image(&mut processor, params, &self.font_atlas)
    }

    /// Set maximum file size for load_image (in bytes)
    #[wasm_bindgen]
    pub fn set_max_file_size(&mut self, bytes: usize) {
        self.max_file_size = bytes;
    }

    /// Export current result to multiple formats
    #[wasm_bindgen]
    pub fn export(
        &self,
        grid: &AsciiGrid,
        params: &ExportParams,
    ) -> Result<JsExportResult, JsValue> {
        export_all(grid, params)
    }

    /// Export as PNG
    #[wasm_bindgen]
    pub fn export_png(&self, grid: &AsciiGrid) -> Result<Vec<u8>, JsValue> {
        let rgba = grid.rgba().unwrap_or_default();
        let width_px = grid.width_px();
        let height_px = grid.height_px();
        export::png::export_png(&rgba, width_px, height_px)
    }

    /// Export as WebP
    #[wasm_bindgen]
    pub fn export_webp(&self, grid: &AsciiGrid, quality: f32, lossless: bool) -> Result<Vec<u8>, JsValue> {
        let rgba = grid.rgba().unwrap_or_default();
        let width_px = grid.width_px();
        let height_px = grid.height_px();
        export::webp::export_webp(&rgba, width_px, height_px, quality, lossless)
    }

    /// Export as plain text
    #[wasm_bindgen]
    pub fn export_text(&self, grid: &AsciiGrid) -> String {
        let cols = grid.cols();
        let rows = grid.rows();
        export::text::export_text(&grid.data(), cols, rows)
    }

    /// Export as HTML with embedded font (base64 WOFF)
    #[wasm_bindgen]
    pub fn export_html(
        &self,
        grid: &AsciiGrid,
        fg_color: &str,
        bg_color: &str,
        font_family: &str,
        line_height: f32,
        font_size: u32,
    ) -> String {
        let cols = grid.cols();
        let rows = grid.rows();
        // Get font variant name for the title
        export::html::export_html(
            &grid.data(), cols, rows,
            fg_color, bg_color, font_family,
            line_height, font_size,
            self.font_variant
        )
    }

    /// Export as ANSI escape codes
    #[wasm_bindgen]
    pub fn export_ansi(&self, grid: &AsciiGrid, fg_rgb: u32, bg_rgb: u32) -> String {
        let cols = grid.cols();
        let rows = grid.rows();
        export::ansi::export_ansi(&grid.data(), cols, rows, fg_rgb, bg_rgb)
    }
}