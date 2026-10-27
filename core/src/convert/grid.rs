//! ASCII grid data structure

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone, Debug, PartialEq)]
pub struct AsciiGrid {
    data: Vec<u16>,
    cols: u32,
    rows: u32,
    rgba: Option<Vec<u8>>,
}

#[wasm_bindgen]
impl AsciiGrid {
    #[wasm_bindgen(constructor)]
    pub fn new(cols: u32, rows: u32) -> AsciiGrid {
        AsciiGrid {
            data: vec![0; (cols * rows) as usize],
            cols,
            rows,
            rgba: None,
        }
    }

    pub fn cols(&self) -> u32 { self.cols }
    pub fn rows(&self) -> u32 { self.rows }
    pub fn len(&self) -> usize { self.data.len() }

    pub fn get(&self, col: u32, row: u32) -> u16 {
        let idx = (row * self.cols + col) as usize;
        self.data.get(idx).copied().unwrap_or(0)
    }

    pub fn set(&mut self, col: u32, row: u32, value: u16) {
        let idx = (row * self.cols + col) as usize;
        if idx < self.data.len() {
            self.data[idx] = value;
        }
    }

    pub fn data(&self) -> Vec<u16> { self.data.clone() }

    pub fn set_rgba(&mut self, rgba: Vec<u8>) {
        self.rgba = Some(rgba);
    }

    pub fn rgba(&self) -> Option<Vec<u8>> { self.rgba.clone() }

    pub fn width_px(&self) -> u32 { self.cols * 8 }
    pub fn height_px(&self) -> u32 { self.rows * 8 }
}

impl AsciiGrid {
    pub fn iter(&self) -> impl Iterator<Item = (u32, u32, u16)> + '_ {
        self.data.iter().enumerate().map(move |(i, &v)| {
            let col = (i % self.cols as usize) as u32;
            let row = (i / self.cols as usize) as u32;
            (col, row, v)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_grid() {
        let grid = AsciiGrid::new(10, 5);
        assert_eq!(grid.cols(), 10);
        assert_eq!(grid.rows(), 5);
        assert_eq!(grid.len(), 50);
        assert_eq!(grid.width_px(), 80);
        assert_eq!(grid.height_px(), 40);
    }

    #[test]
    fn test_get_set() {
        let mut grid = AsciiGrid::new(4, 3);
        grid.set(0, 0, 1);
        grid.set(3, 2, 15);
        
        assert_eq!(grid.get(0, 0), 1);
        assert_eq!(grid.get(3, 2), 15);
        assert_eq!(grid.get(1, 1), 0); // Default
    }

    #[test]
    fn test_get_oob() {
        let grid = AsciiGrid::new(2, 2);
        assert_eq!(grid.get(5, 5), 0);
        assert_eq!(grid.get(0, 5), 0);
        assert_eq!(grid.get(5, 0), 0);
    }

    #[test]
    fn test_set_oob_ignored() {
        let mut grid = AsciiGrid::new(2, 2);
        grid.set(5, 5, 10); // Should be ignored
        assert_eq!(grid.data.len(), 4);
    }

    #[test]
    fn test_data_clone() {
        let mut grid = AsciiGrid::new(2, 2);
        grid.set(0, 0, 1);
        grid.set(1, 1, 2);
        
        let data = grid.data();
        assert_eq!(data, vec![1, 0, 0, 2]);
        
        // Modifying clone shouldn't affect original
        // (data() returns a new Vec)
    }

    #[test]
    fn test_rgba() {
        let mut grid = AsciiGrid::new(2, 2);
        assert_eq!(grid.rgba(), None);
        
        let rgba = vec![255u8; 16];
        grid.set_rgba(rgba.clone());
        assert_eq!(grid.rgba(), Some(rgba));
    }

    #[test]
    fn test_iter() {
        let mut grid = AsciiGrid::new(2, 2);
        grid.set(0, 0, 1);
        grid.set(1, 0, 2);
        grid.set(0, 1, 3);
        grid.set(1, 1, 4);
        
        let items: Vec<_> = grid.iter().collect();
        assert_eq!(items.len(), 4);
        assert!(items.contains(&(0, 0, 1)));
        assert!(items.contains(&(1, 0, 2)));
        assert!(items.contains(&(0, 1, 3)));
        assert!(items.contains(&(1, 1, 4)));
    }

    #[test]
    fn test_iter_order() {
        let mut grid = AsciiGrid::new(3, 2);
        for row in 0..2 {
            for col in 0..3 {
                grid.set(col, row, (row * 3 + col) as u16);
            }
        }
        
        let data = grid.data();
        // Row-major order
        assert_eq!(data, vec![0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_empty_grid() {
        let grid = AsciiGrid::new(0, 0);
        assert_eq!(grid.len(), 0);
        assert_eq!(grid.data.len(), 0);
    }

    #[test]
    fn test_width_height_px() {
        let grid = AsciiGrid::new(10, 5);
        assert_eq!(grid.width_px(), 80);
        assert_eq!(grid.height_px(), 40);
    }
}