//! 24-bit ANSI export with per-cell foreground/background colors.
//!
//! Only our own generated escape sequences appear in the output: every
//! character comes from the fixed ramp of the selected variant (printable
//! ASCII block glyphs — never control characters), and colors are produced
//! exclusively from validated u8 cell arrays as decimal `38;2`/`48;2`
//! sequences. Untrusted bytes can never be injected as escapes.

use crate::error::{CoreError, Result};
use crate::font::VariantFont;
use crate::limits::checked_cells;

pub fn export_ansi(
    grid: &[u16],
    fg: &[u8],
    bg: &[u8],
    cols: u32,
    rows: u32,
    font: &VariantFont,
) -> Result<String> {
    let cells = checked_cells(cols, rows)?;
    if grid.len() != cells {
        return Err(CoreError::export(format!(
            "grid has {} entries, expected {cells}",
            grid.len()
        )));
    }
    if fg.len() != cells * 4 || bg.len() != cells * 4 {
        return Err(CoreError::export(format!(
            "cell color arrays must be {} bytes (got fg={} bg={})",
            cells * 4,
            fg.len(),
            bg.len()
        )));
    }

    let mut out = String::with_capacity(cells * 24 + 32);
    let mut cur_fg: Option<[u8; 3]> = None;
    let mut cur_bg: Option<[u8; 3]> = None;

    for row in 0..rows {
        if row > 0 {
            out.push('\n');
            // Newline resets nothing; colors carry over per row like ANSI art.
        }
        for col in 0..cols {
            let idx = (row * cols + col) as usize;
            let f = [fg[idx * 4], fg[idx * 4 + 1], fg[idx * 4 + 2]];
            let b = [bg[idx * 4], bg[idx * 4 + 1], bg[idx * 4 + 2]];
            if cur_fg != Some(f) {
                out.push_str(&format!("\x1b[38;2;{};{};{}m", f[0], f[1], f[2]));
                cur_fg = Some(f);
            }
            if cur_bg != Some(b) {
                out.push_str(&format!("\x1b[48;2;{};{};{}m", b[0], b[1], b[2]));
                cur_bg = Some(b);
            }
            out.push(font.glyph_to_char(grid[idx]));
        }
    }
    out.push_str("\x1b[0m");
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::{FontVariant, VariantFont};

    #[test]
    fn per_cell_colors_and_reset() {
        let font = VariantFont::load(FontVariant::Unscii8).unwrap();
        let grid = vec![1u16, 2, 3, 4];
        let fg = vec![
            255, 0, 0, 255, // red
            255, 0, 0, 255, // same -> no repeat
            0, 255, 0, 255, // green
            0, 255, 0, 255,
        ];
        let bg = vec![
            0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 255, 255, 0, 0, 255, 255,
        ];
        let s = export_ansi(&grid, &fg, &bg, 2, 2, &font).unwrap();
        assert!(s.ends_with("\x1b[0m"));
        assert!(s.contains("\x1b[38;2;255;0;0m"));
        assert!(s.contains("\x1b[38;2;0;255;0m"));
        assert!(s.contains("\x1b[48;2;0;0;255m"));
        // Red emitted once for two adjacent cells (change-only optimization).
        assert_eq!(s.matches("\x1b[38;2;255;0;0m").count(), 1);
        let lines: Vec<&str> = s.split('\n').collect();
        assert_eq!(lines.len(), 2);
        // No raw ESC outside our generated sequences: strip known sequences
        // and assert nothing suspicious remains.
        let stripped = s.replace("\x1b[0m", "");
        let mut rest = stripped.as_str();
        while let Some(start) = rest.find('\x1b') {
            let end = rest[start..].find('m').unwrap() + start;
            let seq = &rest[start..=end];
            let ok = seq.starts_with("\x1b[38;2;") || seq.starts_with("\x1b[48;2;");
            assert!(ok, "unexpected sequence {seq:?}");
            rest = &rest[end + 1..];
        }
    }

    #[test]
    fn validates_buffer_lengths() {
        let font = VariantFont::load(FontVariant::Unscii8).unwrap();
        assert!(export_ansi(&[0, 1, 2, 3], &[0u8; 4], &[0u8; 16], 2, 2, &font).is_err());
    }
}
