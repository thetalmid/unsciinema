//! ANSI escape code export for terminal display

use crate::convert::mapper::glyph_to_char;

pub fn export_ansi(grid: &[u16], cols: u32, rows: u32, fg: u32, bg: u32) -> String {
    let fg_r = ((fg >> 24) & 0xFF) as u8;
    let fg_g = ((fg >> 16) & 0xFF) as u8;
    let fg_b = ((fg >> 8) & 0xFF) as u8;
    let bg_r = ((bg >> 24) & 0xFF) as u8;
    let bg_g = ((bg >> 16) & 0xFF) as u8;
    let bg_b = ((bg >> 8) & 0xFF) as u8;
    
    let mut out = String::new();
    out.push_str(&format!("\x1b[38;2;{};{};{}m\x1b[48;2;{};{};{}m", 
        fg_r, fg_g, fg_b, bg_r, bg_g, bg_b));
    
    for row in 0..rows {
        for col in 0..cols {
            let idx = grid[(row * cols + col) as usize] as usize;
            out.push(glyph_to_char(idx as u16));
        }
        if row < rows - 1 {
            out.push('\n');
        }
    }
    
    out.push_str("\x1b[0m"); // Reset
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ansi_export_basic() {
        let grid = vec![0, 15, 15, 0];
        let ansi = export_ansi(&grid, 2, 2, 0xFFFFFFFF, 0x000000FF);
        assert!(ansi.contains("\x1b[38;2;255;255;255m"));
        assert!(ansi.contains("\x1b[48;2;0;0;0m"));
        assert!(ansi.contains("\x1b[0m"));
    }

    #[test]
    fn test_ansi_export_fg_bg() {
        let grid = vec![0];
        let ansi = export_ansi(&grid, 1, 1, 0xFF0000FF, 0x00FF00FF);
        assert!(ansi.contains("\x1b[38;2;255;0;0m"));
        assert!(ansi.contains("\x1b[48;2;0;255;0m"));
    }

    #[test]
    fn test_ansi_reset_at_end() {
        let grid = vec![0];
        let ansi = export_ansi(&grid, 1, 1, 0xFFFFFFFF, 0x000000FF);
        assert!(ansi.ends_with("\x1b[0m"));
    }

    #[test]
    fn test_ansi_multiline() {
        let grid = vec![0, 1, 2, 3];
        let ansi = export_ansi(&grid, 2, 2, 0xFFFFFFFF, 0x000000FF);
        let lines: Vec<&str> = ansi.lines().collect();
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn test_ansi_pack_unpack() {
        // Red = 0xFF0000FF
        let packed: u32 = 0xFF0000FF;
        let r = ((packed >> 24) & 0xFF) as u8;
        let g = ((packed >> 16) & 0xFF) as u8;
        let b = ((packed >> 8) & 0xFF) as u8;
        let a = (packed & 0xFF) as u8;
        assert_eq!((r, g, b, a), (255, 0, 0, 255));
    }

    #[test]
    fn test_ansi_different_colors() {
        let grid = vec![15; 4]; // Full blocks
        let ansi = export_ansi(&grid, 2, 2, 0x00FF00FF, 0xFF0000FF);
        assert!(ansi.contains("█"));
        assert!(ansi.contains("\x1b[38;2;0;255;0m")); // Green fg
        assert!(ansi.contains("\x1b[48;2;255;0;0m")); // Red bg
    }

    #[test]
    fn test_ansi_black_on_white() {
        let grid = vec![0, 15];
        let ansi = export_ansi(&grid, 2, 1, 0x000000FF, 0xFFFFFFFF);
        assert!(ansi.contains("\x1b[38;2;0;0;0m"));
        assert!(ansi.contains("\x1b[48;2;255;255;255m"));
    }
}