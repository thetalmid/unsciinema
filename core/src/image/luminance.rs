//! Luminance conversion (BT.709 standard for sRGB)

/// Convert RGBA to luminance using BT.709 coefficients
/// L = 0.2126*R + 0.7152*G + 0.0722*B
pub fn rgba_to_luma(rgba: &[u8], width: u32, height: u32) -> Vec<u8> {
    let pixel_count = (width * height) as usize;
    let mut luma = Vec::with_capacity(pixel_count);
    
    for chunk in rgba.chunks_exact(4) {
        let r = chunk[0] as f32;
        let g = chunk[1] as f32;
        let b = chunk[2] as f32;
        let l = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        luma.push(l.clamp(0.0, 255.0) as u8);
    }
    luma
}

/// Convert RGBA to luminance (in-place, returns luma buffer)
pub fn rgba_to_luma_inplace(rgba: &[u8]) -> Vec<u8> {
    rgba.chunks_exact(4)
        .map(|chunk| {
            let r = chunk[0] as f32;
            let g = chunk[1] as f32;
            let b = chunk[2] as f32;
            (0.2126 * r + 0.7152 * g + 0.0722 * b).clamp(0.0, 255.0) as u8
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_luma_white() {
        let rgba = vec![255, 255, 255, 255];
        let luma = rgba_to_luma_inplace(&rgba);
        assert_eq!(luma[0], 255);
    }

    #[test]
    fn test_luma_black() {
        let rgba = vec![0, 0, 0, 255];
        let luma = rgba_to_luma_inplace(&rgba);
        assert_eq!(luma[0], 0);
    }

    #[test]
    fn test_luma_red() {
        // Pure red: 0.2126 * 255 ≈ 54.2
        let rgba = vec![255, 0, 0, 255];
        let luma = rgba_to_luma_inplace(&rgba);
        assert!((53..=55).contains(&luma[0]), "Red luma = {}", luma[0]);
    }

    #[test]
    fn test_luma_green() {
        // Pure green: 0.7152 * 255 ≈ 182.4
        let rgba = vec![0, 255, 0, 255];
        let luma = rgba_to_luma_inplace(&rgba);
        assert!((181..=183).contains(&luma[0]), "Green luma = {}", luma[0]);
    }

    #[test]
    fn test_luma_blue() {
        // Pure blue: 0.0722 * 255 ≈ 18.4
        let rgba = vec![0, 0, 255, 255];
        let luma = rgba_to_luma_inplace(&rgba);
        assert!((17..=20).contains(&luma[0]), "Blue luma = {}", luma[0]);
    }

    #[test]
    fn test_luma_gray() {
        // Equal RGB = 128: 0.2126*128 + 0.7152*128 + 0.0722*128 = 128
        let rgba = vec![128, 128, 128, 255];
        let luma = rgba_to_luma_inplace(&rgba);
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
        let luma = rgba_to_luma_inplace(&rgba);
        assert_eq!(luma.len(), 4);
        assert!((53..=55).contains(&luma[0]));
        assert!((181..=183).contains(&luma[1]));
        assert!((17..=20).contains(&luma[2]));
        assert_eq!(luma[3], 255);
    }

    #[test]
    fn test_rgba_to_luma_with_dimensions() {
        let rgba = vec![255, 255, 255, 255].repeat(4); // 2x2 white
        let luma = rgba_to_luma(&rgba, 2, 2);
        assert_eq!(luma.len(), 4);
        assert_eq!(luma[0], 255);
    }

    #[test]
    fn test_luma_clamping() {
        // Values that could exceed 255 due to floating point
        let rgba = vec![255, 255, 255, 255];
        let luma = rgba_to_luma_inplace(&rgba);
        assert!(luma[0] <= 255);
    }
}