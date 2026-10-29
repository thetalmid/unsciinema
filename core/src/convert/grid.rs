//! `AsciiGrid`: checked row-major glyph-index grid.

use crate::error::{CoreError, Result};
use crate::limits::checked_cells;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsciiGrid {
    cols: u32,
    rows: u32,
    /// Row-major, length == cols * rows, values are atlas indices (NOT codepoints).
    data: Vec<u16>,
}

impl AsciiGrid {
    pub fn new(cols: u32, rows: u32) -> Result<AsciiGrid> {
        let cells = checked_cells(cols, rows)?;
        Ok(AsciiGrid {
            cols,
            rows,
            data: vec![0u16; cells],
        })
    }

    /// Construct from precomputed levels with the same length validation.
    /// (Production uses the checked `set` path; this constructor is pinned by
    /// tests and available to native tooling.)
    #[allow(dead_code)]
    pub fn from_levels(cols: u32, rows: u32, levels: Vec<u16>) -> Result<AsciiGrid> {
        let cells = checked_cells(cols, rows)?;
        if levels.len() != cells {
            return Err(CoreError::invalid(format!(
                "grid data has {} entries, expected {cells}",
                levels.len()
            )));
        }
        Ok(AsciiGrid {
            cols,
            rows,
            data: levels,
        })
    }

    pub fn cols(&self) -> u32 {
        self.cols
    }

    pub fn rows(&self) -> u32 {
        self.rows
    }

    /// Total cells (== cols * rows).
    /// Validated by tests; kept as the checked accessor of record.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    fn offset(&self, col: u32, row: u32) -> Result<usize> {
        if col >= self.cols || row >= self.rows {
            return Err(CoreError::invalid(format!(
                "cell ({col},{row}) outside {}x{}",
                self.cols, self.rows
            )));
        }
        Ok((row * self.cols + col) as usize)
    }

    /// Checked read accessor (tests + callers outside the WASM lib build).
    #[allow(dead_code)]
    pub fn get(&self, col: u32, row: u32) -> Result<u16> {
        Ok(self.data[self.offset(col, row)?])
    }

    pub fn set(&mut self, col: u32, row: u32, v: u16) -> Result<()> {
        let i = self.offset(col, row)?;
        self.data[i] = v;
        Ok(())
    }

    pub fn indices(&self) -> &[u16] {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_indexing() {
        let mut g = AsciiGrid::new(4, 3).unwrap();
        g.set(3, 2, 15).unwrap();
        assert_eq!(g.get(3, 2).unwrap(), 15);
        assert!(g.get(4, 0).is_err());
        assert!(g.get(0, 3).is_err());
    }

    #[test]
    fn rejects_bad_shapes() {
        assert!(AsciiGrid::new(0, 5).is_err());
        assert!(AsciiGrid::new(5, 0).is_err());
        let too_big = checked_cells(4_000_001, 1);
        assert!(too_big.is_err());
        assert!(AsciiGrid::from_levels(2, 2, vec![1, 2, 3]).is_err());
    }
}
