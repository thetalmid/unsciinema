//! Plain UTF-8 text export: exactly the grid, row by row.
//!
//! Limitation (surfaced in the UI): text cannot embed the font, colors, CRT
//! effects, or raster blending — it carries characters only.

use crate::error::{CoreError, Result};
use crate::font::VariantFont;
use crate::limits::checked_cells;

pub fn export_text(grid: &[u16], cols: u32, rows: u32, font: &VariantFont) -> Result<String> {
    let cells = checked_cells(cols, rows)?;
    if grid.len() != cells {
        return Err(CoreError::export(format!(
            "grid has {} entries, expected {cells}",
            grid.len()
        )));
    }
    // One line per row, no trailing newline: splitting the output on '\n'
    // yields exactly `rows` strings, and trailing spaces in rows are kept.
    let mut out = String::with_capacity(cells + rows as usize);
    for row in 0..rows {
        if row > 0 {
            out.push('\n');
        }
        for col in 0..cols {
            let idx = grid[(row * cols + col) as usize];
            out.push(font.glyph_to_char(idx));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::{FontVariant, VariantFont};

    #[test]
    fn exact_rows_and_spaces_preserved() {
        let font = VariantFont::load(FontVariant::Unscii8).unwrap();
        // 3x2: [space, dot, space] / [hash, space, space]
        let grid = vec![0u16, 1, 0, 15, 0, 0];
        let s = export_text(&grid, 3, 2, &font).unwrap();
        let lines: Vec<&str> = s.split('\n').collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].chars().count(), 3);
        assert_eq!(lines[0], format!(" {} ", font.glyph_to_char(1)));
        assert_eq!(lines[1].chars().count(), 3);
        // Trailing spaces preserved.
        assert!(lines[1].ends_with("  "));
        assert_eq!(lines[1].chars().next().unwrap(), font.glyph_to_char(15));
    }

    #[test]
    fn validates_lengths() {
        let font = VariantFont::load(FontVariant::Unscii8).unwrap();
        assert!(export_text(&[0, 1, 2], 2, 2, &font).is_err());
        assert!(export_text(&[0], 1, 0, &font).is_err());
    }
}
