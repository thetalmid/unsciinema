//! Plain text export

use crate::convert::mapper::glyph_to_char;

pub fn export_text(grid: &[u16], cols: u32, rows: u32) -> String {
    let mut out = String::with_capacity((cols + 1) as usize * rows as usize);
    
    for row in 0..rows {
        for col in 0..cols {
            let idx = grid[(row * cols + col) as usize] as usize;
            out.push(glyph_to_char(idx as u16));
        }
        if row < rows - 1 {
            out.push('\n');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_export_basic() {
        let grid = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let text = export_text(&grid, 4, 4);
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 4);
        assert_eq!(lines[0].chars().count(), 4);
        assert_eq!(lines[3].chars().count(), 4);
    }

    #[test]
    fn test_text_export_contains_expected_chars() {
        let grid = vec![0, 1, 2, 3];
        let text = export_text(&grid, 2, 2);
        assert!(text.contains(' '));
        assert!(text.contains('.'));
        assert!(text.contains(':'));
        assert!(text.contains(','));
    }

    #[test]
    fn test_text_export_newlines() {
        let grid = vec![0, 1, 2, 3];
        let text = export_text(&grid, 2, 2);
        assert_eq!(text.matches('\n').count(), 1);
    }

    #[test]
    fn test_text_export_no_trailing_newline() {
        let grid = vec![0, 1];
        let text = export_text(&grid, 1, 2);
        assert!(!text.ends_with('\n'));
    }

    #[test]
    fn test_text_export_1x1() {
        let grid = vec![15]; // Full block
        let text = export_text(&grid, 1, 1);
        assert_eq!(text, "█");
    }

    #[test]
    fn test_text_export_large() {
        let grid = vec![7u16; 100]; // 10x10 mid-gray
        let text = export_text(&grid, 10, 10);
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 10);
        for line in lines {
            assert_eq!(line.chars().count(), 10);
        }
    }
}