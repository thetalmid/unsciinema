//! Color palettes for multi-color dithering

/// Predefined color palettes (packed RGBA u32)
/// Predefined color palettes (packed RGBA u32, format 0xRRGGBBAA)
/// Most palettes are ordered dark-to-light; rainbow is hue-ordered.
/// Alpha is always 0xFF (opaque) for consistent rendering.
pub const PALETTES: &[(&str, &[u32])] = &[
    // Monochrome
    ("monochrome", &[0x000000FF, 0xFFFFFFFF]),
    ("terminal", &[0x000000FF, 0x333333FF, 0x666666FF, 0x999999FF, 0xCCCCCCFF, 0xFFFFFFFF]),
    ("gameboy", &[0x0F380FFF, 0x306230FF, 0x8BAC0FFF, 0x9BBC0FFF]),

    // Warm
    ("sepia", &[0x1A1A1AFF, 0x4A3A2AFF, 0x8A6A4AFF, 0xD4B896FF, 0xF5E6C8FF]),
    ("campfire", &[0x0D0D0DFF, 0x3D1A0DFF, 0x8B3A1AFF, 0xD47A3AFF, 0xF5C896FF]),
    ("gold", &[0x0D0D00FF, 0x3D3D00FF, 0xB8B800FF, 0xFFF000FF, 0xFFFF80FF]),
    ("autumn", &[0x1A0D00FF, 0x4A1A00FF, 0x8B3A00FF, 0xD46A00FF, 0xF5A600FF]),

    // Cool
    ("ocean", &[0x000D1AFF, 0x003D4DFF, 0x007A8CFF, 0x4DB8D4FF, 0x96E6F5FF]),
    ("ice", &[0x00001AFF, 0x002A6DFF, 0x006AB8FF, 0x4DA6E6FF, 0x96D4F5FF]),
    ("midnight", &[0x00001AFF, 0x001A4DFF, 0x003A8CFF, 0x4A6AD4FF, 0x9696F5FF]),

    // Neon / Synthwave
    ("synthwave", &[0x1A0033FF, 0x4D0066FF, 0xFF00FFFF, 0xFF66FFFF, 0xFFCCFFFF]),
    ("cyberpunk", &[0x0D001AFF, 0x3D0066FF, 0xB800FFFF, 0xFF00FFFF, 0xFF80FFFF]),
    ("neon", &[0x00001AFF, 0x004D00FF, 0x00FF00FF, 0x00FF80FF, 0x00FFFFFF]),
    ("matrix", &[0x000D00FF, 0x003300FF, 0x00AA00FF, 0x00FF00FF, 0x80FF80FF]),

    // RGB - ordered by luminance (BT.709): black, blue, red, magenta, green, cyan, yellow, white
    ("rgb", &[0x000000FF, 0x0000FFFF, 0xFF0000FF, 0xFF00FFFF, 0x00FF00FF, 0x00FFFFFF, 0xFFFF00FF, 0xFFFFFFFF]),
    // CMYK - ordered dark to light: black, magenta, cyan, yellow, white
    ("cmyk", &[0x000000FF, 0xFF00FFFF, 0x00FFFFFF, 0xFFFF00FF, 0xFFFFFFFF]),

    // Artistic
    ("ink", &[0x000000FF, 0x1A1A1AFF, 0x333333FF, 0x666666FF, 0xCCCCCCFF]),
    // Pastel - ordered dark to light: 8080B0, B0B0D0, FFB0FF, B0E0B0, B0FFFF, FFFFB0
    ("pastel", &[0x8080B0FF, 0xB0B0D0FF, 0xFFB0FFFF, 0xB0E0B0FF, 0xB0FFFFFF, 0xFFFFB0FF]),
    ("vintage", &[0x2A1A0DFF, 0x4A3A1AFF, 0x8A6A3AFF, 0xCAB88AFF, 0xEDE8D4FF]),
    ("noir", &[0x000000FF, 0x1A1A1AFF, 0x333333FF, 0x4A4A4AFF, 0x666666FF]),

    // High contrast
    // (high-contrast is identical to monochrome; kept for compatibility)
    ("high-contrast", &[0x000000FF, 0xFFFFFFFF]),
    ("red-scale", &[0x000000FF, 0x330000FF, 0x660000FF, 0x990000FF, 0xCC0000FF, 0xFF0000FF]),
    ("green-scale", &[0x000000FF, 0x003300FF, 0x006600FF, 0x009900FF, 0x00CC00FF, 0x00FF00FF]),
    ("blue-scale", &[0x000000FF, 0x000033FF, 0x000066FF, 0x000099FF, 0x0000CCFF, 0x0000FFFF]),

    // Special
    ("rainbow", &[0xFF0000FF, 0xFF7F00FF, 0xFFFF00FF, 0x00FF00FF, 0x0000FFFF, 0x4B0082FF, 0x9400D3FF]),
    ("fire", &[0x000000FF, 0x330000FF, 0x661100FF, 0x993300FF, 0xCC5500FF, 0xFF7700FF, 0xFF9900FF, 0xFFCC00FF]),
    ("electric", &[0x00001AFF, 0x00004DFF, 0x000080FF, 0x0000B3FF, 0x0000E6FF, 0x0033FFFF, 0x0066FFFF, 0x0099FFFF]),
    ("royal", &[0x1A0033FF, 0x330066FF, 0x4D0099FF, 0x6600CCFF, 0x8000FFFF, 0x9933FFFF, 0xB266FFFF, 0xCC99FFFF]),
    ("forest", &[0x001A00FF, 0x003300FF, 0x004D00FF, 0x006600FF, 0x008000FF, 0x009900FF, 0x00B300FF, 0x00CC00FF]),
    ("sunset", &[0x330000FF, 0x661100FF, 0x993300FF, 0xCC5500FF, 0xFF7700FF, 0xFF9933FF, 0xFFBB66FF, 0xFFDD99FF]),
    ("ocean-deep", &[0x000022FF, 0x000044FF, 0x001166FF, 0x002288FF, 0x0033AAFF, 0x0044CCFF, 0x0055EEFF, 0x0066FFFF]),
];

/// Get palette by name (used by tests; presets are also exported as JSON
/// from lib.rs so the UI reads a single source of truth).
#[allow(dead_code)]
pub fn get_palette(name: &str) -> Option<&[u32]> {
    PALETTES.iter()
        .find(|(n, _)| *n == name)
        .map(|(_, p)| *p)
}

/// Get all palette names.
#[allow(dead_code)]
pub fn palette_names() -> Vec<&'static str> {
    PALETTES.iter().map(|(n, _)| *n).collect()
}

/// Pack RGBA components into u32 (0xRRGGBBAA).
/// Kept from the salvage as the documented inverse of `unpack_rgba`; tests
/// pin its shift behavior.
#[allow(dead_code)]
pub fn pack_rgba(r: u8, g: u8, b: u8, a: u8) -> u32 {
    ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | (a as u32)
}

/// Unpack u32 to RGBA components
pub fn unpack_rgba(packed: u32) -> (u8, u8, u8, u8) {
    (
        (packed >> 24) as u8,
        (packed >> 16) as u8,
        (packed >> 8) as u8,
        packed as u8,
    )
}

// ---------------------------------------------------------------------------
// Validation and nearest-color mapping (added during reintegration).
// ---------------------------------------------------------------------------

use crate::error::{CoreError, Result};

/// Custom/preset palette constraints: at least 1 entry (the UI asks for 2+),
/// at most 64 (enough for any preset; nearest search stays trivial).
pub const MAX_PALETTE_COLORS: usize = 64;

pub fn validate_palette(colors: &[u32]) -> Result<()> {
    if colors.is_empty() {
        return Err(CoreError::invalid("palette has no colors"));
    }
    if colors.len() > MAX_PALETTE_COLORS {
        return Err(CoreError::limit(format!(
            "palette has {} colors (max {MAX_PALETTE_COLORS})",
            colors.len()
        )));
    }
    Ok(())
}

/// Nearest palette entry by "redmean" (a common perceptual approximation),
/// ignoring alpha — palette ORDER is never assumed luminance-sorted (rainbow
/// is intentionally hue-ordered), so color distance is the only safe mapping.
pub fn nearest_index(palette: &[u32], r: u8, g: u8, b: u8) -> usize {
    let mut best = 0usize;
    let mut best_d = i64::MAX;
    for (i, &c) in palette.iter().enumerate() {
        let (pr, pg, pb, _) = unpack_rgba(c);
        let dr = pr as i64 - r as i64;
        let dg = pg as i64 - g as i64;
        let db = pb as i64 - b as i64;
        let rm = (pr as i64 + r as i64) / 2;
        let d = (2 + rm / 256) * dr * dr + 4 * dg * dg + (2 + (255 - rm) / 256) * db * db;
        if d < best_d {
            best_d = d;
            best = i;
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exactly_31_presets() {
        assert_eq!(PALETTES.len(), 31);
        assert_eq!(palette_names().len(), 31);
    }

    #[test]
    fn several_presets_have_eight_colors() {
        let eight = PALETTES
            .iter()
            .filter(|(_, p)| p.len() >= 8)
            .map(|(n, _)| *n)
            .collect::<Vec<_>>();
        // rgb, fire, electric, royal, ocean-deep all ship 8 entries.
        assert!(eight.len() >= 4, "got {eight:?}");
        assert!(eight.contains(&"rgb"));
        assert!(eight.contains(&"rainbow") == false || PALETTES.iter().any(|(n, p)| *n == "rainbow" && p.len() == 7));
    }

    #[test]
    fn rainbow_keeps_hue_order_not_luminance_order() {
        let rainbow = get_palette("rainbow").unwrap();
        assert_eq!(
            rainbow,
            &[
                0xFF0000FF, 0xFF7F00FF, 0xFFFF00FF, 0x00FF00FF, 0x0000FFFF, 0x4B0082FF, 0x9400D3FF
            ]
        );
        // Red first, green (highest BT.709 luma among these) third -> not sorted by luminance.
        assert_ne!(rainbow[0], 0x00FF00FF);
    }

    #[test]
    fn pack_unpack_roundtrip_is_shift_based() {
        assert_eq!(pack_rgba(0x12, 0x34, 0x56, 0x78), 0x12345678);
        assert_eq!(unpack_rgba(0x12345678), (0x12, 0x34, 0x56, 0x78));
        // Endianness trap check: byte order must be R,G,B,A regardless of host.
        let packed = pack_rgba(255, 0, 128, 255);
        let bytes = packed.to_be_bytes();
        assert_eq!(bytes, [255, 0, 128, 255]);
    }

    #[test]
    fn nearest_maps_exact_and_ignores_order() {
        let p = [0xFF0000FFu32, 0x00FF00FF, 0x0000FFFF];
        assert_eq!(nearest_index(&p, 255, 0, 0), 0);
        assert_eq!(nearest_index(&p, 0, 255, 0), 1);
        assert_eq!(nearest_index(&p, 0, 0, 255), 2);
        assert_eq!(nearest_index(&p, 0, 0, 0), 0); // tie -> first
    }

    #[test]
    fn validate_limits() {
        assert!(validate_palette(&[0u32; 1]).is_ok());
        assert!(validate_palette(&[]).is_err());
        assert!(validate_palette(&[0u32; 65]).is_err());
        assert!(validate_palette(&[0u32; 64]).is_ok());
    }
}
