//! Color palettes for multi-color dithering

/// Predefined color palettes (packed RGBA u32)
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
    ("neon", &[0x00001AFF, 0x004D00FF, 0x00FF00FF, 0x00FF80FF, 0x80FFFF80]),
    ("matrix", &[0x000D00FF, 0x003300FF, 0x00AA00FF, 0x00FF00FF, 0x80FF80FF]),
    
    // RGB
    ("rgb", &[0x000000FF, 0xFF0000FF, 0x00FF00FF, 0x0000FFFF, 0xFFFF00FF, 0xFF00FFFF, 0x00FFFFFF, 0xFFFFFFFF]),
    ("cmyk", &[0xFFFFFFFF, 0x00FFFFFF, 0xFF00FFFF, 0xFFFF00FF, 0x000000FF]),
    
    // Artistic
    ("ink", &[0x000000FF, 0x1A1A1AFF, 0x333333FF, 0x666666FF, 0xCCCCCCFF]),
    ("pastel", &[0xFFE0E0FF, 0xFFE0FFFF, 0xE0FFE0FF, 0xE0FFFFE0, 0xFFFFE0FF, 0xFFE0FFFF]),
    ("vintage", &[0x2A1A0DFF, 0x4A3A1AFF, 0x8A6A3AFF, 0xCAB88AFF, 0xEDE8D4FF]),
    ("noir", &[0x000000FF, 0x1A1A1AFF, 0x333333FF, 0x4A4A4AFF, 0x666666FF]),
    
    // High contrast
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

/// Get palette by name
pub fn get_palette(name: &str) -> Option<&[u32]> {
    PALETTES.iter()
        .find(|(n, _)| *n == name)
        .map(|(_, p)| *p)
}

/// Get all palette names
pub fn palette_names() -> Vec<&'static str> {
    PALETTES.iter().map(|(n, _)| *n).collect()
}

/// Pack RGBA components into u32 (0xRRGGBBAA)
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

/// Convert hex color string to packed RGBA
/// Parse comma-separated hex colors into palette

#[cfg(test)]
mod test_utils {
    use super::*;

    /// Parse hex color string to RGBA u32 (test utility)
    pub fn hex_to_rgba(hex: &str) -> Option<u32> {
        let hex = hex.trim_start_matches('#');
        let (r, g, b, a) = if hex.len() == 6 {
            (
                u8::from_str_radix(&hex[0..2], 16).ok()?,
                u8::from_str_radix(&hex[2..4], 16).ok()?,
                u8::from_str_radix(&hex[4..6], 16).ok()?,
                255,
            )
        } else if hex.len() == 8 {
            (
                u8::from_str_radix(&hex[0..2], 16).ok()?,
                u8::from_str_radix(&hex[2..4], 16).ok()?,
                u8::from_str_radix(&hex[4..6], 16).ok()?,
                u8::from_str_radix(&hex[6..8], 16).ok()?,
            )
        } else {
            return None;
        };
        Some(pack_rgba(r, g, b, a))
    }

    /// Parse comma-separated hex colors into palette (test utility)
    pub fn parse_palette(colors: &str) -> Vec<u32> {
        colors.split(',')
            .filter_map(|s| hex_to_rgba(s.trim()))
            .collect()
    }
}
#[cfg(test)]
mod tests {
    use crate::dither::palette::test_utils::{hex_to_rgba, parse_palette};

    use super::*;

    #[test]
    fn test_palette_loading() {
        assert!(get_palette("gameboy").is_some());
        assert!(get_palette("synthwave").is_some());
        assert!(get_palette("terminal").is_some());
        assert!(get_palette("nonexistent").is_none());
    }

    #[test]
    fn test_palette_names() {
        let names = palette_names();
        assert!(names.contains(&"gameboy"));
        assert!(names.contains(&"synthwave"));
        assert!(names.contains(&"monochrome"));
        assert!(names.len() >= 31);
    }

    #[test]
    fn test_gameboy_palette() {
        let palette = get_palette("gameboy").unwrap();
        assert_eq!(palette.len(), 4);
        assert_eq!(palette[0], 0x0F380FFF);
        assert_eq!(palette[3], 0x9BBC0FFF);
    }

    #[test]
    fn test_terminal_palette() {
        let palette = get_palette("terminal").unwrap();
        assert_eq!(palette.len(), 6);
        assert_eq!(palette[0], 0x000000FF);
        assert_eq!(palette[5], 0xFFFFFFFF);
    }

    #[test]
    fn test_pack_unpack() {
        let packed = pack_rgba(255, 128, 64, 255);
        let (r, g, b, a) = unpack_rgba(packed);
        assert_eq!((r, g, b, a), (255, 128, 64, 255));
    }

    #[test]
    fn test_pack_unpack_zero() {
        let packed = pack_rgba(0, 0, 0, 0);
        let (r, g, b, a) = unpack_rgba(packed);
        assert_eq!((r, g, b, a), (0, 0, 0, 0));
    }

    #[test]
    fn test_pack_unpack_max() {
        let packed = pack_rgba(255, 255, 255, 255);
        let (r, g, b, a) = unpack_rgba(packed);
        assert_eq!((r, g, b, a), (255, 255, 255, 255));
    }

    #[test]
    fn test_hex_parse_6_digit() {
        assert_eq!(hex_to_rgba("#FF0000"), Some(pack_rgba(255, 0, 0, 255)));
        assert_eq!(hex_to_rgba("#00FF00"), Some(pack_rgba(0, 255, 0, 255)));
        assert_eq!(hex_to_rgba("#0000FF"), Some(pack_rgba(0, 0, 255, 255)));
    }

    #[test]
    fn test_hex_parse_8_digit() {
        assert_eq!(hex_to_rgba("#FF0000FF"), Some(pack_rgba(255, 0, 0, 255)));
        assert_eq!(hex_to_rgba("#FF000080"), Some(pack_rgba(255, 0, 0, 128)));
        assert_eq!(hex_to_rgba("#00000000"), Some(pack_rgba(0, 0, 0, 0)));
    }

    #[test]
    fn test_hex_parse_invalid() {
        assert!(hex_to_rgba("invalid").is_none());
        assert!(hex_to_rgba("#FF00").is_none());
        assert!(hex_to_rgba("#GGGGGG").is_none());
    }

    #[test]
    fn test_parse_palette() {
        let palette = parse_palette("#FF0000, #00FF00, #0000FF");
        assert_eq!(palette.len(), 3);
        assert_eq!(palette[0], pack_rgba(255, 0, 0, 255));
        assert_eq!(palette[1], pack_rgba(0, 255, 0, 255));
        assert_eq!(palette[2], pack_rgba(0, 0, 255, 255));
    }

    #[test]
    fn test_parse_palette_with_spaces() {
        let palette = parse_palette("#FF0000, #00FF00 , #0000FF");
        assert_eq!(palette.len(), 3);
    }

    #[test]
    fn test_parse_palette_invalid_skipped() {
        let palette = parse_palette("#FF0000, invalid, #00FF00");
        assert_eq!(palette.len(), 2);
    }

    #[test]
    fn test_palette_count() {
        assert_eq!(PALETTES.len(), 31);
    }

    #[test]
    fn test_all_palettes_have_colors() {
        for (name, colors) in PALETTES {
            assert!(!colors.is_empty(), "Palette {} is empty", name);
            assert!(colors.len() >= 2, "Palette {} has < 2 colors", name);
            assert!(colors.len() <= 8, "Palette {} has > 8 colors", name);
        }
    }
}