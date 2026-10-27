//! Font handling: unscii .hex parsing, atlas generation, variant management

pub mod parser;
pub mod atlas;
pub mod variants;

pub use crate::font::atlas::FontAtlas;
pub use crate::font::variants::FontVariant;
pub use crate::font::variants::CHAR_RAMP;
use crate::font::parser::{parse_unscii_hex, parse_unscii_hex_ramp, ParseError};
use wasm_bindgen::prelude::*;

/// Load a font variant by name and generate its atlas
pub fn load_font_variant(variant: FontVariant) -> Result<FontAtlas, JsValue> {
    let hex_data = variant.hex_data();
    let all_glyphs = parse_unscii_hex(hex_data).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let ramp_glyphs = parse_unscii_hex_ramp(hex_data, &crate::font::variants::CHAR_RAMP)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    FontAtlas::from_glyphs(ramp_glyphs, variant)
}