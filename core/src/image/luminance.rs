//! Luminance and alpha handling.
//!
//! BT.709 coefficients are applied to *encoded* (non-linear) sRGB channel
//! values — a luma-like `Y'`, not linear-light luminance. This matches the
//! salvaged code and is labelled accordingly everywhere.

/// Alpha handling modes (mission contract, implemented end-to-end).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlphaMode {
    /// 0 — Ignore source alpha entirely: luma/colors come from RGB as stored;
    /// output alpha comes only from the configured background / opaque ink.
    Ignore = 0,
    /// 1 — Preserve transparency: source alpha drives per-cell background
    /// alpha (and ink alpha); fully transparent pixels contribute the
    /// background color to luma so they map to sparse glyphs.
    Preserve = 1,
    /// 2 — Composite source over the configured background first; output
    /// alpha equals the configured background alpha (source alpha consumed).
    Blend = 2,
}

impl AlphaMode {
    pub fn from_u32(v: u32) -> Option<AlphaMode> {
        match v {
            0 => Some(AlphaMode::Ignore),
            1 => Some(AlphaMode::Preserve),
            2 => Some(AlphaMode::Blend),
            _ => None,
        }
    }
}

/// BT.709 luma on encoded sRGB channels, result in [0, 255].
#[inline]
pub fn bt709_luma(r: f32, g: f32, b: f32) -> f32 {
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// Composite `src` (straight RGBA) over `bg` (straight RGBA), straight result.
#[inline]
pub fn over_straight(sr: f32, sg: f32, sb: f32, sa: f32, br: f32, bg: f32, bb: f32, ba: f32) -> (f32, f32, f32, f32) {
    let ia = 1.0 - sa;
    let a = sa + ba * ia;
    if a <= 0.0 {
        (0.0, 0.0, 0.0, 0.0)
    } else {
        (
            (sr * sa + br * ba * ia) / a,
            (sg * sa + bg * ba * ia) / a,
            (sb * sa + bb * ba * ia) / a,
            a,
        )
    }
}

/// Per-pixel luma under an alpha mode, RGB returned after the alpha stage.
///
/// - `Ignore`: RGB used as stored (alpha fully discarded).
/// - `Preserve`: RGB as stored; fully transparent pixels use the background
///   color for luma purposes (their own alpha is still carried separately).
/// - `Blend`: RGB already composited over background upstream; used as stored.
#[inline]
pub fn stage_rgb_luma(
    r: f32,
    g: f32,
    b: f32,
    a: f32,
    mode: AlphaMode,
    bg: [f32; 3],
) -> (f32, f32, f32) {
    match mode {
        AlphaMode::Ignore => (r, g, b),
        AlphaMode::Preserve => {
            if a <= 0.0 {
                (bg[0], bg[1], bg[2])
            } else {
                (r, g, b)
            }
        }
        AlphaMode::Blend => (r, g, b),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bt709_known_values() {
        assert!((bt709_luma(255.0, 255.0, 255.0) - 255.0).abs() < 0.01);
        assert!((bt709_luma(0.0, 0.0, 0.0)).abs() < 0.01);
        // Green contributes most.
        let g = bt709_luma(0.0, 255.0, 0.0);
        let r = bt709_luma(255.0, 0.0, 0.0);
        assert!(g > r);
    }

    #[test]
    fn over_composites_correctly() {
        // Opaque red over opaque black -> red opaque.
        let (r, g, b, a) = over_straight(255.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0);
        assert_eq!((r, g, b, a), (255.0, 0.0, 0.0, 1.0));
        // Transparent source keeps background.
        let (r, g, b, a) = over_straight(10.0, 20.0, 30.0, 0.0, 5.0, 6.0, 7.0, 1.0);
        assert_eq!((r, g, b, a), (5.0, 6.0, 7.0, 1.0));
        // Half red over half black: alpha 0.5*1 + 1*0.5 = 1... straight over
        // of sa=0.5 onto ba=1 => a = 0.5 + 1*0.5 = 1, color = mix.
        let (r, _, _, a) = over_straight(255.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 1.0);
        assert_eq!(a, 1.0);
        assert!((r - 127.5).abs() < 0.01);
    }

    #[test]
    fn preserve_uses_bg_for_fully_transparent_luma() {
        let bg = [0.0, 0.0, 0.0];
        let (r, g, b) = stage_rgb_luma(255.0, 255.0, 255.0, 0.0, AlphaMode::Preserve, bg);
        assert_eq!((r, g, b), (0.0, 0.0, 0.0));
        // Partial alpha keeps stored RGB (documented).
        let (r, g, b) = stage_rgb_luma(100.0, 150.0, 200.0, 0.4, AlphaMode::Preserve, bg);
        assert_eq!((r, g, b), (100.0, 150.0, 200.0));
        // Ignore keeps stored RGB even at alpha 0 (documented).
        let (r, g, b) = stage_rgb_luma(255.0, 0.0, 0.0, 0.0, AlphaMode::Ignore, bg);
        assert_eq!((r, g, b), (255.0, 0.0, 0.0));
    }
}
