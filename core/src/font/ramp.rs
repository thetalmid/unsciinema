//! Per-variant character ramp: sparse -> dense, measured from real glyphs.
//!
//! There is exactly ONE ramp definition in the codebase: `BASE_RAMP`, the
//! salvaged 16-character set that spans the full tonal range (space -> full
//! block) for unscii-8. For every other variant the SAME 16 characters are
//! re-ordered by their measured ink density in that variant's glyph data,
//! which repairs density inversions between styles (where e.g. `*` and `%`
//! swap weight). Ties keep the base order (stable), so the mapping is
//! deterministic on every platform.
//!
//! Everyone (atlas, quantizer, text/HTML/ANSI exports, WebGL, CPU raster)
//! reads the resulting `ramp` from the conversion result, so the
//! index -> character mapping can never drift between surfaces.
//!
//! `space` always measures 0 ink and `█` always measures 1.0 in these fonts,
//! so the ramp always covers both endpoints (asserted at load).

use crate::error::{CoreError, Result};
use crate::font::parser::{parse_glyphs_for, Glyph};
use crate::font::variants::FontVariant;

/// The canonical ramp set (salvaged from `convert/mapper.rs` /
/// `font/variants.rs` — the two salvaged copies agreed byte-for-byte).
/// This is the single source of truth; do not duplicate it anywhere else.
pub const BASE_RAMP: [char; 16] = [
    ' ', '.', ',', '-', ':', '=', '+', '%', '*', '8', '&', '@', 'W', 'M', '#', '█',
];

/// Ramp length used by the quantizer/atlas.
pub const RAMP_LEN: usize = 16;

/// Compute the sparse->dense ordering of `BASE_RAMP` for one variant.
pub fn compute_ramp(hex: &str, variant: FontVariant) -> Result<Vec<char>> {
    // Determine expected dimensions from '.', the safest ubiquitous probe.
    let probe = parse_glyphs_for(hex, &['.'], None)?;
    let expected = probe
        .get(&'.')
        .map(|g| (g.width, g.height))
        .ok_or_else(|| CoreError::font(format!("{}: missing probe glyph '.'", variant.name())))?;

    let glyphs = parse_glyphs_for(hex, &BASE_RAMP, Some(expected))?;
    if glyphs.len() != RAMP_LEN {
        // All seven shipped variants contain the full set; a failure here
        // means the font file is not a supported Unscii variant.
        let missing: Vec<char> = BASE_RAMP
            .iter()
            .filter(|c| !glyphs.contains_key(c))
            .copied()
            .collect();
        return Err(CoreError::font(format!(
            "{}: missing ramp glyphs {missing:?}",
            variant.name()
        )));
    }

    // (density, base order index, char)
    let mut scored: Vec<(f64, usize, char)> = BASE_RAMP
        .iter()
        .enumerate()
        .map(|(i, &ch)| {
            let g: &Glyph = &glyphs[&ch];
            (g.ink_density(), i, ch)
        })
        .collect();

    // Stable sort by density; ties keep the base order.
    scored.sort_by(|a, b| {
        a.0.partial_cmp(&b.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.1.cmp(&b.1))
    });

    let ramp: Vec<char> = scored.iter().map(|s| s.2).collect();

    // Verified invariants: space first, full block last, non-decreasing.
    if ramp.first() != Some(&' ') {
        return Err(CoreError::font(format!(
            "{}: space is not the sparsest glyph",
            variant.name()
        )));
    }
    if ramp.last() != Some(&'█') {
        return Err(CoreError::font(format!(
            "{}: full block is not the densest glyph",
            variant.name()
        )));
    }
    for w in scored.windows(2) {
        if w[1].0 < w[0].0 {
            return Err(CoreError::font(format!(
                "{}: ramp not density ordered",
                variant.name()
            )));
        }
    }
    Ok(ramp)
}

/// Measured densities of a ramp (verification tooling; exercised by the
/// all-variants test and reported in INTEGRATION.md).
#[allow(dead_code)]
pub fn ramp_densities(hex: &str, ramp: &[char]) -> Result<Vec<f64>> {
    let probe = parse_glyphs_for(hex, &['.'], None)?;
    let expected = probe.get(&'.').map(|g| (g.width, g.height));
    let glyphs = parse_glyphs_for(hex, ramp, expected)?;
    let mut out = Vec::with_capacity(ramp.len());
    for &ch in ramp {
        let g = glyphs
            .get(&ch)
            .ok_or_else(|| CoreError::font(format!("missing ramp glyph {ch:?}")))?;
        out.push(g.ink_density());
    }
    Ok(out)
}
