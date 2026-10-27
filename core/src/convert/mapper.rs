//! Map luminance values to glyph indices

/// Map luma buffer to glyph indices based on density
/// Uses average of all pixels in each cell for better quality
pub fn map_luma_to_glyphs(
    luma: &[u8],
    width: u32,
    height: u32,
    cell_size: f32,
    threshold: f32,
) -> Result<crate::convert::grid::AsciiGrid, wasm_bindgen::JsValue> {
    let cell_size = cell_size.max(1.0);
    let cols = (width as f32 / cell_size).floor() as u32;
    let rows = (height as f32 / cell_size).floor() as u32;
    
    if cols == 0 || rows == 0 {
        return Err(wasm_bindgen::JsValue::from_str("Cell size too large for image"));
    }
    
    let mut grid = crate::convert::grid::AsciiGrid::new(cols, rows);
    
    for row in 0..rows {
        for col in 0..cols {
            // Compute cell bounds in source image
            let cell_x0 = (col as f32 * cell_size) as u32;
            let cell_y0 = (row as f32 * cell_size) as u32;
            let cell_x1 = ((col as f32 + 1.0) * cell_size).floor() as u32;
            let cell_y1 = ((row as f32 + 1.0) * cell_size).floor() as u32;
            
            let cell_x1 = cell_x1.min(width);
            let cell_y1 = cell_y1.min(height);
            
            // Average all pixels in the cell
            let mut sum = 0u32;
            let mut count = 0u32;
            for cy in cell_y0..cell_y1 {
                for cx in cell_x0..cell_x1 {
                    let idx = (cy * width + cx) as usize;
                    sum += luma[idx] as u32;
                    count += 1;
                }
            }
            
            let avg_luma = if count > 0 { sum / count } else { 0 };
            let luma_val = avg_luma as f32 / 255.0;
            
            let glyph_idx = if luma_val < threshold {
                0u16  // Below threshold -> space (sparse)
            } else {
                // Scale from threshold to 1.0 -> glyph 0 to 15
                let t = (luma_val - threshold) / (1.0 - threshold);
                (t * 15.0).round() as u16
            };
            let glyph_idx = glyph_idx.min(15);
            
            grid.set(col, row, glyph_idx);
        }
    }
    
    Ok(grid)
}

/// Get character for glyph index (for text export)
pub fn glyph_to_char(glyph_idx: u16) -> char {
    const CHAR_RAMP: [char; 16] = [
        ' ', '.', ':', ',', '-', '+', '*', '=', '%', '@', '8', '&', 'W', 'M', '#', '█',
    ];
    CHAR_RAMP.get(glyph_idx as usize).copied().unwrap_or(' ')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glyph_to_char() {
        assert_eq!(glyph_to_char(0), ' ');
        assert_eq!(glyph_to_char(15), '█');
    }

    #[test]
    fn test_glyph_to_char_oob() {
        assert_eq!(glyph_to_char(16), ' ');
        assert_eq!(glyph_to_char(100), ' ');
    }

    #[test]
    fn test_map_luma_to_glyphs_basic() {
        // 8x8 image, cell_size=8 -> 1x1 grid
        let luma = vec![200u8; 64]; // ~0.78 brightness
        let grid = map_luma_to_glyphs(&luma, 8, 8, 8.0, 0.5).unwrap();
        assert_eq!(grid.cols(), 1);
        assert_eq!(grid.rows(), 1);
        assert_eq!(grid.get(0, 0), 9); // 200/255 = 0.78 -> glyph 9
    }

    #[test]
    fn test_map_luma_to_glyphs_white() {
        let luma = vec![255u8; 64];
        let grid = map_luma_to_glyphs(&luma, 8, 8, 8.0, 0.5).unwrap();
        assert_eq!(grid.get(0, 0), 15); // White -> dense glyph
    }

    #[test]
    fn test_map_luma_to_glyphs_black() {
        let luma = vec![0u8; 64];
        let grid = map_luma_to_glyphs(&luma, 8, 8, 8.0, 0.5).unwrap();
        assert_eq!(grid.get(0, 0), 0); // Black -> space
    }

    #[test]
    fn test_map_luma_to_glyphs_grid() {
        // 16x16 image, cell_size=8 -> 2x2 grid
        let luma = vec![128u8; 256];
        let grid = map_luma_to_glyphs(&luma, 16, 16, 8.0, 0.5).unwrap();
        assert_eq!(grid.cols(), 2);
        assert_eq!(grid.rows(), 2);
        assert_eq!(grid.len(), 4);
    }

    #[test]
    fn test_map_luma_threshold() {
        // Below threshold should map to space
        let luma = vec![100u8; 64]; // 100/255 ≈ 0.39
        let grid = map_luma_to_glyphs(&luma, 8, 8, 8.0, 0.5).unwrap();
        assert_eq!(grid.get(0, 0), 0); // Below 0.5 threshold -> space
    }

    #[test]
    fn test_map_luma_cell_size() {
        // 16x16 image, cell_size=4 -> 4x4 grid
        let luma = vec![128u8; 256];
        let grid = map_luma_to_glyphs(&luma, 16, 16, 4.0, 0.5).unwrap();
        assert_eq!(grid.cols(), 4);
        assert_eq!(grid.rows(), 4);
        assert_eq!(grid.len(), 16);
    }

    #[test]
    fn test_map_luma_cell_size_fractional() {
        // 10x10 image, cell_size=3 -> 3x3 grid (floor)
        let luma = vec![128u8; 100];
        let grid = map_luma_to_glyphs(&luma, 10, 10, 3.0, 0.5).unwrap();
        assert_eq!(grid.cols(), 3);
        assert_eq!(grid.rows(), 3);
    }

    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_map_luma_cell_size_too_large() {
        let luma = vec![128u8; 64];
        let result = map_luma_to_glyphs(&luma, 8, 8, 16.0, 0.5);
        assert!(result.is_err());
    }

    #[test]
    fn test_map_luma_min_cell_size() {
        let luma = vec![128u8; 64];
        let grid = map_luma_to_glyphs(&luma, 8, 8, 0.5, 0.5).unwrap(); // Clamped to 1.0
        assert_eq!(grid.cols(), 8);
        assert_eq!(grid.rows(), 8);
    }

    #[test]
    fn test_map_luma_gradient() {
        // Create gradient from black to white (8x8)
        let mut luma = Vec::with_capacity(64);
        for i in 0..64 {
            luma.push((i * 4) as u8); // 0, 4, 8, ... 252
        }
        let grid = map_luma_to_glyphs(&luma, 8, 8, 8.0, 0.0).unwrap();
        // Average of gradient -> ~127 -> glyph 7 or 8
        assert!(grid.get(0, 0) >= 7 && grid.get(0, 0) <= 9);
    }

    #[test]
    fn test_map_luma_cell_average() {
        // 16x16, cell_size=8 -> 2x2 grid
        // Top-left quadrant white (255), rest black (0)
        let mut luma = vec![0u8; 256];
        for y in 0..8 {
            for x in 0..8 {
                luma[(y * 16 + x) as usize] = 255;
            }
        }
        let grid = map_luma_to_glyphs(&luma, 16, 16, 8.0, 0.5).unwrap();
        assert_eq!(grid.cols(), 2);
        assert_eq!(grid.rows(), 2);
        // Top-left cell average = 255 -> glyph 15
        assert_eq!(grid.get(0, 0), 15);
        // Other cells average = 0 -> glyph 0
        assert_eq!(grid.get(1, 0), 0);
        assert_eq!(grid.get(0, 1), 0);
        assert_eq!(grid.get(1, 1), 0);
    }
}