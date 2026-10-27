//! unscii font variants and their embedded .hex data

use crate::font::parser::Glyph;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FontVariant {
    Unscii8 = 0,
    Unscii8Alt = 1,
    Unscii8Fantasy = 2,
    Unscii8Mcr = 3,
    Unscii8Tall = 4,
    Unscii8Thin = 5,
    Unscii16 = 6,
}

impl FontVariant {
    pub fn name(&self) -> &'static str {
        match self {
            FontVariant::Unscii8 => "unscii-8",
            FontVariant::Unscii8Alt => "unscii-8-alt",
            FontVariant::Unscii8Fantasy => "unscii-8-fantasy",
            FontVariant::Unscii8Mcr => "unscii-8-mcr",
            FontVariant::Unscii8Tall => "unscii-8-tall",
            FontVariant::Unscii8Thin => "unscii-8-thin",
            FontVariant::Unscii16 => "unscii-16",
        }
    }

    pub fn hex_data(&self) -> &'static str {
        match self {
            FontVariant::Unscii8 => include_str!("../../../public/fonts/unscii-8.hex"),
            FontVariant::Unscii8Alt => include_str!("../../../public/fonts/unscii-8-alt.hex"),
            FontVariant::Unscii8Fantasy => include_str!("../../../public/fonts/unscii-8-fantasy.hex"),
            FontVariant::Unscii8Mcr => include_str!("../../../public/fonts/unscii-8-mcr.hex"),
            FontVariant::Unscii8Tall => include_str!("../../../public/fonts/unscii-8-tall.hex"),
            FontVariant::Unscii8Thin => include_str!("../../../public/fonts/unscii-8-thin.hex"),
            FontVariant::Unscii16 => include_str!("../../../public/fonts/unscii-16.hex"),
        }
    }

    pub fn all() -> &'static [FontVariant] {
        &[
            FontVariant::Unscii8,
            FontVariant::Unscii8Alt,
            FontVariant::Unscii8Fantasy,
            FontVariant::Unscii8Mcr,
            FontVariant::Unscii8Tall,
            FontVariant::Unscii8Thin,
            FontVariant::Unscii16,
        ]
    }
}

/// Density-ordered character ramp for 16-level ASCII (sparse → dense)
pub const CHAR_RAMP: [char; 16] = [
    ' ',      // 0: empty (0% density)
    '.',      // 1: single dot (~3%)
    ':',      // 2: two dots (~6%)
    ',',      // 3: comma (~9%)
    '-',      // 4: horizontal line (~12%)
    '+',      // 5: cross (~25%)
    '*',      // 6: asterisk (~30%)
    '=',      // 7: double line (~35%)
    '%',      // 8: percent (~45%)
    '@',      // 9: at sign (~55%)
    '8',      // 10: eight (~60%)
    '&',      // 11: ampersand (~65%)
    'W',      // 12: W (~70%)
    'M',      // 13: M (~75%)
    '#',      // 14: hash (~85%)
    '█',      // 15: full block (100%)
];

pub const CHAR_RAMP_BLOCKS: [char; 16] = [
    ' ', '░', '▒', '▓', '█', '█', '█', '█', '█', '█', '█', '█', '█', '█', '█', '█',
];

pub const CHAR_RAMP_BRAILLE: [char; 16] = [
    '⠀', '⠁', '⠃', '⠇', '⠏', '⠟', '⠿', '⠿', '⠿', '⠿', '⠿', '⠿', '⠿', '⠿', '⠿', '⠿',
];

pub fn get_ramp_glyphs(all_glyphs: &std::collections::HashMap<char, Glyph>) -> Vec<Glyph> {
    CHAR_RAMP.iter()
        .map(|&ch| all_glyphs.get(&ch).cloned().unwrap_or_else(|| Glyph {
            codepoint: ch,
            width: 8,
            height: 8,
            data: vec![0u8; 8],
        }))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variant_names() {
        assert_eq!(FontVariant::Unscii8.name(), "unscii-8");
        assert_eq!(FontVariant::Unscii16.name(), "unscii-16");
        assert_eq!(FontVariant::Unscii8Alt.name(), "unscii-8-alt");
        assert_eq!(FontVariant::Unscii8Fantasy.name(), "unscii-8-fantasy");
    }

    #[test]
    fn test_char_ramp_length() {
        assert_eq!(CHAR_RAMP.len(), 16);
    }

    #[test]
    fn test_char_ramp_order() {
        // Verify ramp goes from sparse to dense
        assert_eq!(CHAR_RAMP[0], ' ');   // Most sparse
        assert_eq!(CHAR_RAMP[15], '█'); // Most dense
    }

    #[test]
    fn test_all_variants() {
        let variants = FontVariant::all();
        assert_eq!(variants.len(), 7);
    }

    #[test]
    fn test_variant_hex_data_not_empty() {
        for variant in FontVariant::all() {
            let data = variant.hex_data();
            assert!(!data.is_empty(), "Variant {} has empty hex data", variant.name());
            assert!(data.contains(':'), "Variant {} missing hex format", variant.name());
        }
    }

    #[test]
    fn test_block_ramp() {
        assert_eq!(CHAR_RAMP_BLOCKS.len(), 16);
        assert_eq!(CHAR_RAMP_BLOCKS[0], ' ');
        assert_eq!(CHAR_RAMP_BLOCKS[15], '█');
    }

    #[test]
    fn test_braille_ramp() {
        assert_eq!(CHAR_RAMP_BRAILLE.len(), 16);
        assert_eq!(CHAR_RAMP_BRAILLE[0], '⠀');
    }
}