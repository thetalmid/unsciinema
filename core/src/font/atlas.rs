//! Font atlas generation for GPU upload
//!
//! Creates a texture atlas containing 16 glyphs arranged horizontally
//! Supports different glyph dimensions (8×8, 8×16, 16×16, etc.)
//! White (255) = ink, Black (0) = background

use crate::font::parser::Glyph;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone, Debug, PartialEq)]
pub struct FontAtlas {
    #[wasm_bindgen(getter_with_clone)]
    pub width: u32,         // atlas width in pixels
    #[wasm_bindgen(getter_with_clone)]
    pub height: u32,        // atlas height in pixels
    #[wasm_bindgen(getter_with_clone)]
    pub glyph_width: u32,   // single glyph width
    #[wasm_bindgen(getter_with_clone)]
    pub glyph_height: u32,  // single glyph height
    #[wasm_bindgen(getter_with_clone)]
    pub glyph_count: u32,   // 16
    #[wasm_bindgen(getter_with_clone)]
    pub data: Vec<u8>,      // RGBA: width × height × 4
    #[wasm_bindgen(getter_with_clone)]
    pub char_ramp: Vec<u16>,
}

#[wasm_bindgen]
impl FontAtlas {
    /// Create atlas from exactly 16 glyphs (placeholder for wasm-bindgen)
    #[wasm_bindgen(constructor)]
    pub fn new(_glyphs: Vec<JsValue>, _variant: crate::font::variants::FontVariant) -> Result<FontAtlas, JsValue> {
        Err(JsValue::from_str("Use load_font_variant instead"))
    }
}

impl FontAtlas {
    /// Create atlas from exactly 16 glyphs - internal use
    pub(crate) fn from_glyphs(glyphs: Vec<Glyph>, _variant: crate::font::variants::FontVariant) -> Result<FontAtlas, JsValue> {
        if glyphs.len() != 16 {
            return Err(JsValue::from_str(&format!("Expected 16 glyphs, got {}", glyphs.len())));
        }

        // Determine glyph dimensions from the first glyph
        // All glyphs in a variant should have the same dimensions
        let glyph_w = glyphs[0].width;
        let glyph_h = glyphs[0].height;

        // Verify all glyphs have same dimensions
        for glyph in &glyphs {
            if glyph.width != glyph_w || glyph.height != glyph_h {
                return Err(JsValue::from_str(&format!(
                    "All glyphs must have same dimensions, got {}x{} and {}x{}",
                    glyph_w, glyph_h, glyph.width, glyph.height
                )));
            }
        }

        // Atlas: 16 glyphs horizontally
        const GLYPH_COUNT: u32 = 16;
        let atlas_width = GLYPH_COUNT * glyph_w;
        let atlas_height = glyph_h;

        let mut data = vec![0u8; (atlas_width * atlas_height * 4) as usize];

        for (glyph_idx, glyph) in glyphs.iter().enumerate() {
            let base_x = (glyph_idx as u32 * glyph_w) as u32;

            // Glyph data is stored as 1 bit per pixel, row-major
            // Each row is 1 byte (8 pixels), so glyph_h bytes per glyph
            // But glyphs can be wider than 8 pixels!
            // The unscii format uses 1 byte per row regardless of width,
            // with unused bits for wider glyphs (or we need to handle differently)

            for gy in 0..glyph_h {
                for gx in 0..glyph_w {
                    let atlas_x = base_x + gx;
                    let atlas_y = gy;

                    let pixel_idx = ((atlas_y * atlas_width + atlas_x) * 4) as usize;

                    // Calculate bit position in glyph data
                    // For glyphs wider than 8, we need multiple bytes per row
                    let bytes_per_row = glyph_w.div_ceil(8);
                    let byte_idx = (gy * bytes_per_row + gx / 8) as usize;
                    let bit_idx = 7 - (gx % 8);

                    let ink = if byte_idx < glyph.data.len() {
                        (glyph.data[byte_idx] >> bit_idx) & 1 == 1
                    } else {
                        false
                    };

                    let val = if ink { 255u8 } else { 0u8 };
                    data[pixel_idx] = val;     // R
                    data[pixel_idx + 1] = val; // G
                    data[pixel_idx + 2] = val; // B
                    data[pixel_idx + 3] = 255; // A
                }
            }
        }

        let char_ramp = crate::font::variants::CHAR_RAMP.iter().map(|&c| c as u16).collect();

        Ok(FontAtlas {
            width: atlas_width,
            height: atlas_height,
            glyph_width: glyph_w,
            glyph_height: glyph_h,
            glyph_count: GLYPH_COUNT,
            data,
            char_ramp,
        })
    }

    /// Get raw RGBA bytes for GPU texture upload
    pub(crate) fn rgba_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Get dimensions as [width, height]
    pub fn dimensions_array(&self) -> [u32; 2] {
        [self.width, self.height]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::parser::Glyph;
    use crate::font::variants::CHAR_RAMP;

    #[test]
    fn test_atlas_creation_8x8() {
        let glyphs: Vec<Glyph> = (0..16).map(|i| Glyph {
            codepoint: CHAR_RAMP[i],
            width: 8,
            height: 8,
            data: vec![0u8; 8],
        }).collect();

        let atlas = FontAtlas::from_glyphs(glyphs, crate::font::variants::FontVariant::Unscii8).unwrap();

        assert_eq!(atlas.width, 128);
        assert_eq!(atlas.height, 8);
        assert_eq!(atlas.glyph_width, 8);
        assert_eq!(atlas.glyph_height, 8);
        assert_eq!(atlas.glyph_count, 16);
        assert_eq!(atlas.data.len(), 128 * 8 * 4);
    }

    #[test]
    fn test_atlas_creation_8x16() {
        let glyphs: Vec<Glyph> = (0..16).map(|i| Glyph {
            codepoint: CHAR_RAMP[i],
            width: 8,
            height: 16,
            data: vec![0u8; 16],  // 1 byte per row * 16 rows
        }).collect();

        let atlas = FontAtlas::from_glyphs(glyphs, crate::font::variants::FontVariant::Unscii8Tall).unwrap();

        assert_eq!(atlas.width, 128);
        assert_eq!(atlas.height, 16);
        assert_eq!(atlas.glyph_width, 8);
        assert_eq!(atlas.glyph_height, 16);
        assert_eq!(atlas.glyph_count, 16);
        assert_eq!(atlas.data.len(), 128 * 16 * 4);
    }

    #[test]
    fn test_atlas_creation_16x16() {
        let glyphs: Vec<Glyph> = (0..16).map(|i| Glyph {
            codepoint: CHAR_RAMP[i],
            width: 16,
            height: 16,
            data: vec![0u8; 32],  // 2 bytes per row * 16 rows
        }).collect();

        let atlas = FontAtlas::from_glyphs(glyphs, crate::font::variants::FontVariant::Unscii16).unwrap();

        assert_eq!(atlas.width, 256);
        assert_eq!(atlas.height, 16);
        assert_eq!(atlas.glyph_width, 16);
        assert_eq!(atlas.glyph_height, 16);
        assert_eq!(atlas.glyph_count, 16);
        assert_eq!(atlas.data.len(), 256 * 16 * 4);
    }

    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_atlas_rejects_wrong_count() {
        let glyphs: Vec<Glyph> = (0..15).map(|i| Glyph {
            codepoint: CHAR_RAMP[i],
            width: 8,
            height: 8,
            data: vec![0u8; 8],
        }).collect();

        let result = FontAtlas::from_glyphs(glyphs, crate::font::variants::FontVariant::Unscii8);
        assert!(result.is_err());
    }

    #[test]
    fn test_atlas_rejects_different_dimensions() {
        let mut glyphs: Vec<Glyph> = (0..16).map(|i| Glyph {
            codepoint: CHAR_RAMP[i],
            width: 8,
            height: 8,
            data: vec![0u8; 8],
        }).collect();
        glyphs[0] = Glyph {
            codepoint: CHAR_RAMP[0],
            width: 16,
            height: 8,
            data: vec![0u8; 16],
        };

        let result = FontAtlas::from_glyphs(glyphs, crate::font::variants::FontVariant::Unscii8);
        assert!(result.is_err());
    }

    #[test]
    fn test_atlas_ink_pixel() {
        let full_glyph = Glyph {
            codepoint: '█',
            width: 8,
            height: 8,
            data: vec![0xFFu8; 8],
        };

        let mut glyphs: Vec<Glyph> = (0..16).map(|i| Glyph {
            codepoint: CHAR_RAMP[i],
            width: 8,
            height: 8,
            data: vec![0u8; 8],
        }).collect();
        glyphs[15] = full_glyph;

        let atlas = FontAtlas::from_glyphs(glyphs, crate::font::variants::FontVariant::Unscii8).unwrap();

        // Check last glyph area (x=120..127, y=0..7) is all white
        for gy in 0..8 {
            for gx in 120..128 {
                let idx = ((gy * 128 + gx) * 4) as usize;
                assert_eq!(atlas.data[idx], 255, "R at ({}, {})", gx, gy);
                assert_eq!(atlas.data[idx + 1], 255, "G at ({}, {})", gx, gy);
                assert_eq!(atlas.data[idx + 2], 255, "B at ({}, {})", gx, gy);
                assert_eq!(atlas.data[idx + 3], 255, "A at ({}, {})", gx, gy);
            }
        }
    }

    #[test]
    fn test_atlas_background_black() {
        let glyphs: Vec<Glyph> = (0..16).map(|i| Glyph {
            codepoint: CHAR_RAMP[i],
            width: 8,
            height: 8,
            data: vec![0u8; 8],
        }).collect();

        let atlas = FontAtlas::from_glyphs(glyphs, crate::font::variants::FontVariant::Unscii8).unwrap();

        // All pixels should be black (0) with alpha 255
        for pixel in atlas.data.chunks_exact(4) {
            assert_eq!(pixel[0], 0, "R should be 0");
            assert_eq!(pixel[1], 0, "G should be 0");
            assert_eq!(pixel[2], 0, "B should be 0");
            assert_eq!(pixel[3], 255, "A should be 255");
        }
    }

    #[test]
    fn test_atlas_partial_glyph() {
        // Glyph with only top row set
        let partial_glyph = Glyph {
            codepoint: 'T',
            width: 8,
            height: 8,
            data: vec![0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        };

        let mut glyphs: Vec<Glyph> = (0..16).map(|i| Glyph {
            codepoint: CHAR_RAMP[i],
            width: 8,
            height: 8,
            data: vec![0u8; 8],
        }).collect();
        glyphs[0] = partial_glyph;

        let atlas = FontAtlas::from_glyphs(glyphs, crate::font::variants::FontVariant::Unscii8).unwrap();

        // First glyph area (x=0..7, y=0) should be white
        for gx in 0..8 {
            let idx = ((0 * 128 + gx) * 4) as usize;
            assert_eq!(atlas.data[idx], 255, "Top row R at x={}", gx);
            assert_eq!(atlas.data[idx + 1], 255, "Top row G at x={}", gx);
            assert_eq!(atlas.data[idx + 2], 255, "Top row B at x={}", gx);
        }

        // Rest should be black
        for gy in 1..8 {
            for gx in 0..8 {
                let idx = ((gy * 128 + gx) * 4) as usize;
                assert_eq!(atlas.data[idx], 0, "Row {} should be black", gy);
            }
        }
    }

    #[test]
    fn test_atlas_8x16_top_row() {
        // Glyph with only top row set (8x16)
        let partial_glyph = Glyph {
            codepoint: 'T',
            width: 8,
            height: 16,
            data: vec![0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                       0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        };

        let mut glyphs: Vec<Glyph> = (0..16).map(|i| Glyph {
            codepoint: CHAR_RAMP[i],
            width: 8,
            height: 16,
            data: vec![0u8; 16],
        }).collect();
        glyphs[0] = partial_glyph;

        let atlas = FontAtlas::from_glyphs(glyphs, crate::font::variants::FontVariant::Unscii8Tall).unwrap();

        // First glyph area (x=0..7, y=0) should be white
        for gx in 0..8 {
            let idx = ((0 * 128 + gx) * 4) as usize;
            assert_eq!(atlas.data[idx], 255, "Top row R at x={}", gx);
        }

        // Rest should be black
        for gy in 1..16 {
            for gx in 0..8 {
                let idx = ((gy * 128 + gx) * 4) as usize;
                assert_eq!(atlas.data[idx], 0, "Row {} should be black", gy);
            }
        }
    }
}