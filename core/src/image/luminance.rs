//! Luminance conversion (BT.709 standard for sRGB)

/// Alpha handling mode
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlphaMode {
    /// Ignore alpha, compute luminance from RGB only
    Ignore = 0,
    /// Treat fully transparent as background color
    Transparent = 1,
    /// Blend with background color using alpha
    Blend = 2,
}

impl From<u32> for AlphaMode {
    fn from(value: u32) -> Self {
        match value {
            0 => AlphaMode::Ignore,
            1 => AlphaMode::Transparent,
            2 => AlphaMode::Blend,
            _ => AlphaMode::Ignore,
        }
    }
}

/// Convert RGBA to luminance using BT.709 coefficients with alpha handling
/// L = 0.2126*R + 0.7152*G + 0.0722*B
pub fn rgba_to_luma(
    rgba: &[u8],
    width: u32,
    height: u32,
    alpha_mode: AlphaMode,
    bg_color: [u8; 3], // Background RGB
) -> Vec<u8> {
    let pixel_count = (width * height) as usize;
    let mut luma = Vec::with_capacity(pixel_count);
    
    for chunk in rgba.chunks_exact(4) {
        let r = chunk[0] as f32;
        let g = chunk[1] as f32;
        let b = chunk[2] as f32;
        let a = chunk[3] as f32 / 255.0;
        
        let (fr, fg, fb) = match alpha_mode {
            AlphaMode::Ignore => (r, g, b),
            AlphaMode::Transparent => {
                if a == 0.0 {
                    (bg_color[0] as f32, bg_color[1] as f32, bg_color[2] as f32)
                } else {
                    (r, g, b)
                }
            }
            AlphaMode::Blend => {
                let br = bg_color[0] as f32;
                let bg = bg_color[1] as f32;
                let bb = bg_color[2] as f32;
                (r * a + br * (1.0 - a), g * a + bg * (1.0 - a), b * a + bb * (1.0 - a))
            }
        };
        
        let l = 0.2126 * fr + 0.7152 * fg + 0.0722 * fb;
        luma.push(l.clamp(0.0, 255.0) as u8);
    }
    luma
}

/// Convert RGBA to luminance with alpha handling (default background = black)
pub fn rgba_to_luma_inplace(rgba: &[u8], alpha_mode: AlphaMode, bg_color: [u8; 3]) -> Vec<u8> {
    rgba.chunks_exact(4)
        .map(|chunk| {
            let r = chunk[0] as f32;
            let g = chunk[1] as f32;
            let b = chunk[2] as f32;
            let a = chunk[3] as f32 / 255.0;
            
            let (fr, fg, fb) = match alpha_mode {
                AlphaMode::Ignore => (r, g, b),
                AlphaMode::Transparent => {
                    if a == 0.0 {
                        (bg_color[0] as f32, bg_color[1] as f32, bg_color[2] as f32)
                    } else {
                        (r, g, b)
                    }
                }
                AlphaMode::Blend => {
                    let br = bg_color[0] as f32;
                    let bg = bg_color[1] as f32;
                    let bb = bg_color[2] as f32;
                    (r * a + br * (1.0 - a), g * a + bg * (1.0 - a), b * a + bb * (1.0 - a))
                }
            };
            
            (0.2126 * fr + 0.7152 * fg + 0.0722 * fb).clamp(0.0, 255.0) as u8
        })
        .collect()
}

// Backward compatible functions (ignore alpha, black background)
pub fn rgba_to_luma_default(rgba: &[u8], width: u32, height: u32) -> Vec<u8> {
    rgba_to_luma(rgba, width, height, AlphaMode::Ignore, [0, 0, 0])
}

pub fn rgba_to_luma_inplace_default(rgba: &[u8]) -> Vec<u8> {
    rgba_to_luma_inplace(rgba, AlphaMode::Ignore, [0, 0, 0])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_luma_white() {
        let rgba = vec![255, 255, 255, 255];
        let luma = rgba_to_luma_inplace_default(&rgba);
        assert_eq!(luma[0], 255);
    }

    #[test]
    fn test_luma_black() {
        let rgba = vec![0, 0, 0, 255];
        let luma = rgba_to_luma_inplace_default(&rgba);
        assert_eq!(luma[0], 0);
    }

    #[test]
    fn test_luma_red() {
        // Pure red: 0.2126 * 255 ≈ 54.2
        let rgba = vec![255, 0, 0, 255];
        let luma = rgba_to_luma_inplace_default(&rgba);
        assert!((53..=55).contains(&luma[0]), "Red luma = {}", luma[0]);
    }

    #[test]
    fn test_luma_green() {
        // Pure green: 0.7152 * 255 ≈ 182.4
        let rgba = vec![0, 255, 0, 255];
        let luma = rgba_to_luma_inplace_default(&rgba);
        assert!((181..=183).contains(&luma[0]), "Green luma = {}", luma[0]);
    }

    #[test]
    fn test_luma_blue() {
        // Pure blue: 0.0722 * 255 ≈ 18.4
        let rgba = vec![0, 0, 255, 255];
        let luma = rgba_to_luma_inplace_default(&rgba);
        assert!((17..=20).contains(&luma[0]), "Blue luma = {}", luma[0]);
    }

    #[test]
    fn test_luma_gray() {
        // Equal RGB = 128: 0.2126*128 + 0.7152*128 + 0.0722*128 = 128
        let rgba = vec![128, 128, 128, 255];
        let luma = rgba_to_luma_inplace_default(&rgba);
        assert_eq!(luma[0], 128);
    }

    #[test]
    fn test_luma_multiple_pixels() {
        let rgba = vec![
            255, 0, 0, 255,   // Red
            0, 255, 0, 255,   // Green
            0, 0, 255, 255,   // Blue
            255, 255, 255, 255, // White
        ];
        let luma = rgba_to_luma_inplace_default(&rgba);
        assert_eq!(luma.len(), 4);
        assert!((53..=55).contains(&luma[0]));
        assert!((181..=183).contains(&luma[1]));
        assert!((17..=20).contains(&luma[2]));
        assert_eq!(luma[3], 255);
    }

    #[test]
    fn test_rgba_to_luma_with_dimensions() {
        let rgba = vec![255, 255, 255, 255].repeat(4); // 2x2 white
        let luma = rgba_to_luma_default(&rgba, 2, 2);
        assert_eq!(luma.len(), 4);
        assert_eq!(luma[0], 255);
    }

    #[test]
    fn test_luma_clamping() {
        // Values that could exceed 255 due to floating point
        let rgba = vec![255, 255, 255, 255];
        let luma = rgba_to_luma_inplace_default(&rgba);
        assert!(luma[0] <= 255);
    }

    #[test]
    fn test_alpha_mode_transparent() {
        // Transparent pixel (alpha=0) with white background should be white
        let rgba = vec![0, 0, 0, 0];
        let luma = rgba_to_luma_inplace(&rgba, AlphaMode::Transparent, [255, 255, 255]);
        assert_eq!(luma[0], 255);
    }

    #[test]
    fn test_alpha_mode_blend() {
        // Semi-transparent red (alpha=128) blended with black = darker red
        let rgba = vec![255, 0, 0, 128];
        let luma = rgba_to_luma_inplace(&rgba, AlphaMode::Blend, [0, 0, 0]);
        // 255 * 0.5 = 127.5 -> 0.2126 * 127.5 ≈ 27
        assert!((26..=28).contains(&luma[0]));
    }

    #[test]
    fn test_alpha_mode_ignore() {
        // Ignore alpha should use RGB values directly
        let rgba = vec![255, 0, 0, 0]; // Transparent red
        let luma = rgba_to_luma_inplace(&rgba, AlphaMode::Ignore, [0, 0, 0]);
        assert!((53..=55).contains(&luma[0]));
    }
}