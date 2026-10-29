//! Font loading: `.hex` parsing + per-variant ramp + atlas, cached per variant.

pub mod atlas;
pub mod parser;
pub mod ramp;
pub mod variants;

pub use atlas::FontAtlas;
pub use variants::FontVariant;

use crate::error::{CoreError, Result};

/// A fully-loaded variant: atlas for its ramp + the ramp itself.
#[derive(Debug, Clone)]
pub struct VariantFont {
    pub variant: FontVariant,
    pub atlas: FontAtlas,
    /// Index -> Unicode character (sparse -> dense). Single source of truth
    /// for glyphIndices -> char mapping in text/HTML/ANSI exports.
    pub ramp: Vec<char>,
}

impl VariantFont {
    pub fn load(variant: FontVariant) -> Result<VariantFont> {
        let hex = variant.hex_data();
        let ramp = ramp::compute_ramp(hex, variant)?;
        let glyphs = parser::parse_glyphs_for(hex, &ramp, None)?;
        let ordered: Vec<parser::Glyph> = ramp
            .iter()
            .map(|ch| {
                glyphs
                    .get(ch)
                    .cloned()
                    .ok_or_else(|| CoreError::font(format!("missing glyph {ch:?}")))
            })
            .collect::<Result<_>>()?;
        // All ramp glyphs must share dimensions for the atlas layout.
        let atlas = FontAtlas::from_glyphs(&ordered)?;
        Ok(VariantFont {
            variant,
            atlas,
            ramp,
        })
    }

    pub fn glyph_to_char(&self, idx: u16) -> char {
        self.ramp
            .get(idx as usize)
            .copied()
            .unwrap_or(' ')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mission-critical verification: every shipped variant gets a 16-entry
    /// sparse->dense ramp measured from its real glyph data (density
    /// inversions between variants are handled per-variant), space is first,
    /// all glyphs share the variant's dimensions, and the atlas matches.
    #[test]
    fn all_variants_have_verified_density_ordered_ramps() {
        for variant in FontVariant::ALL {
            let font = VariantFont::load(variant).unwrap();
            assert_eq!(font.ramp.len(), 16, "{variant:?}");
            assert_eq!(font.ramp[0], ' ', "{variant:?} must start with space");
            assert_eq!(font.atlas.glyph_count, 16);
            let densities =
                ramp::ramp_densities(variant.hex_data(), &font.ramp).unwrap();
            assert_eq!(densities.len(), 16);
            for w in densities.windows(2) {
                assert!(
                    w[1] >= w[0] - 1e-12,
                    "{variant:?} ramp not density ordered: {densities:?}"
                );
            }
            // Atlas geometry: 16 glyphs horizontally, dims of the variant.
            let (gw, gh) = (font.atlas.glyph_width, font.atlas.glyph_height);
            assert!(gw == 8 && (gh == 8 || gh == 16), "{variant:?}: {gw}x{gh}");
            assert_eq!(font.atlas.width, 16 * gw);
            assert_eq!(font.atlas.height, gh);
            assert_eq!(font.atlas.data.len(), (16 * gw * gh * 4) as usize);
            // Ramp glyph chars are all printable/non-control.
            for &c in &font.ramp {
                assert!(!c.is_control(), "{variant:?}: control char {c:?} in ramp");
            }
        }
    }

    /// The base variant's computed ramp must agree with the salvaged ramp
    /// (the salvage was measured on unscii-8) — guards against regression.
    #[test]
    fn base_variant_matches_salvaged_ramp() {
        let font = VariantFont::load(FontVariant::Unscii8).unwrap();
        // Salvage: ' ', '.', ',', '-', ':', '=', '+', '%', '*', '8', '&', '@', 'W', 'M', '#', '█'
        let salvaged = vec![
            ' ', '.', ',', '-', ':', '=', '+', '%', '*', '8', '&', '@', 'W', 'M', '#', '█',
        ];
        assert_eq!(font.ramp, salvaged, "base ramp drifted from salvage");
    }
}
