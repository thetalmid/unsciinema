//! unscii .hex format parser
//!
//! Format: CODEPOINT:HEXBITMAP
//! - CODEPOINT: 5-digit hex Unicode codepoint (e.g., 00041 = 'A')
//! - HEXBITMAP: hex string, length determines dimensions:
//!   - 16 chars = 8×8 (64 bits = 8 bytes)
//!   - 32 chars = 8×16 (128 bits = 16 bytes)
//!   - 64 chars = 16×16 (256 bits = 32 bytes)
//!
//! Bit order: MSB first, left-to-right, top-to-bottom

use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Invalid line format: {0}")]
    InvalidLine(String),
    #[error("Invalid codepoint: {0}")]
    InvalidCodepoint(String),
    #[error("Invalid hex: {0}")]
    InvalidHex(#[from] hex::FromHexError),
    #[error("Unsupported glyph size: {0} bytes")]
    UnsupportedSize(usize),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Glyph {
    pub codepoint: char,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,  // Raw bitmap bytes (1 bit per pixel, MSB first)
}

impl Glyph {
    /// Get pixel at (x, y) - returns true if ink (foreground)
    pub fn get_pixel(&self, x: u32, y: u32) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }
        // Each row is width bits, packed into bytes (8 bits per byte)
        // For 8x8 glyphs, each row is 1 byte
        let byte_index = y * (self.width / 8);
        let bit_index = 7 - x;
        if byte_index < self.data.len() as u32 {
            (self.data[byte_index as usize] >> bit_index) & 1 == 1
        } else {
            false
        }
    }
}

/// Parse unscii .hex format content
pub fn parse_unscii_hex(content: &str) -> Result<HashMap<char, Glyph>, ParseError> {
    let mut glyphs = HashMap::new();

    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let (cp_str, hex_str) = line.split_once(':')
            .ok_or_else(|| ParseError::InvalidLine(format!("line {}: missing ':'", line_num + 1)))?;

        let codepoint = u32::from_str_radix(cp_str.trim(), 16)
            .map_err(|_| ParseError::InvalidCodepoint(cp_str.to_string()))?;

        let ch = char::from_u32(codepoint)
            .ok_or_else(|| ParseError::InvalidCodepoint(format!("invalid unicode: {codepoint}")))?;

        let bytes = hex::decode(hex_str.trim())?;

        let (width, height) = match bytes.len() {
            8 => (8, 8),
            16 => (8, 16),
            32 => (16, 16),
            n => return Err(ParseError::UnsupportedSize(n)),
        };

        glyphs.insert(ch, Glyph {
            codepoint: ch,
            width,
            height,
            data: bytes,
        });
    }

    Ok(glyphs)
}

/// Parse only the glyphs needed for the character ramp
pub fn parse_unscii_hex_ramp(content: &str, ramp: &[char]) -> Result<Vec<Glyph>, ParseError> {
    let all_glyphs = parse_unscii_hex(content)?;
    let mut result = Vec::with_capacity(ramp.len());
    for &ch in ramp {
        result.push(all_glyphs.get(&ch).cloned().ok_or_else(|| ParseError::InvalidLine(format!("Character '{}' not found in font", ch)))?);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_HEX: &str = r#"00020:0000000000000000
0002E:0000000000001818
00041:00007C12127C1010
00042:7C12127C12127C00"#;

    #[test]
    fn test_parse_basic() {
        let glyphs = parse_unscii_hex(TEST_HEX).unwrap();
        assert_eq!(glyphs.len(), 4);
    }

    #[test]
    fn test_parse_space() {
        let glyphs = parse_unscii_hex(TEST_HEX).unwrap();
        let space = glyphs.get(&' ').unwrap();
        assert_eq!(space.codepoint, ' ');
        assert_eq!(space.width, 8);
        assert_eq!(space.height, 8);
        assert_eq!(space.data.len(), 8);
        // Space should be all zeros
        assert!(space.data.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_parse_period() {
        let glyphs = parse_unscii_hex(TEST_HEX).unwrap();
        let period = glyphs.get(&'.').unwrap();
        // Period: 0000000000001818
        // Row 6: 0x18 = 00011000 -> pixels 3,4
        // Row 7: 0x18 = 00011000 -> pixels 3,4
        assert!(period.get_pixel(3, 6), "period center left");
        assert!(period.get_pixel(4, 6), "period center right");
        assert!(period.get_pixel(3, 7), "period bottom left");
        assert!(period.get_pixel(4, 7), "period bottom right");
    }

    #[test]
    fn test_parse_A() {
        let glyphs = parse_unscii_hex(TEST_HEX).unwrap();
        let a = glyphs.get(&'A').unwrap();
        // 00007C12127C1010
        // Row 0: 0x00
        // Row 1: 0x00
        // Row 2: 0x7C = 01111100 -> bits 1-5 set (x=1,2,3,4,5)
        // Row 3: 0x12 = 00010010 -> bits 3, 6 set (x=3, 6)
        // Row 4: 0x12 = 00010010 -> bits 3, 6 set (x=3, 6)
        // Row 5: 0x7C = 01111100 -> bits 1-5 set (x=1,2,3,4,5)
        // Row 6: 0x10 = 00010000 -> bit 3 set (x=3)
        // Row 7: 0x10 = 00010000 -> bit 3 set (x=3)
        
        // Row 2: 0x7C = 01111100, bits 6,5,4,3,2 set -> x=1,2,3,4,5
        assert!(a.get_pixel(2, 2), "A top bar left (x=2)");
        assert!(a.get_pixel(5, 2), "A top bar right (x=5)");
        // Row 3: 0x12 = 00010010, bits 4,1 set -> x=3,6
        assert!(a.get_pixel(3, 3), "A upper left");
        assert!(a.get_pixel(6, 3), "A upper right");
        // Row 5: 0x7C = 01111100, bits 6,5,4,3,2 set -> x=1,2,3,4,5
        assert!(a.get_pixel(2, 5), "A middle bar left (x=2)");
        assert!(a.get_pixel(5, 5), "A middle bar right (x=5)");
        // Row 6: 0x10 = 00010000, bit 4 set -> x=3
        assert!(a.get_pixel(3, 6), "A leg");
    }

    #[test]
    fn test_parse_B() {
        let glyphs = parse_unscii_hex(TEST_HEX).unwrap();
        let b = glyphs.get(&'B').unwrap();
        // 7C12127C12127C00
        // Row 0: 0x7C = 01111100 -> bits 1-5 set (x=1,2,3,4,5)
        // Row 1: 0x12 = 00010010 -> bits 3, 6 set (x=3, 6)
        assert!(b.get_pixel(2, 0), "B top left (x=2)");
        assert!(b.get_pixel(5, 0), "B top right (x=5)");
        assert!(b.get_pixel(3, 1), "B upper left");
    }

    #[test]
    fn test_parse_skips_comments_and_empty() {
        let content = r#"
# Comment
00020:0000000000000000

0002E:0000000000001818
"#;
        let glyphs = parse_unscii_hex(content).unwrap();
        assert_eq!(glyphs.len(), 2);
    }

    #[test]
    fn test_parse_ramp() {
        let ramp = [' ', '.', 'A', 'B'];
        let glyphs = parse_unscii_hex_ramp(TEST_HEX, &ramp).unwrap();
        assert_eq!(glyphs.len(), 4);
        assert_eq!(glyphs[0].codepoint, ' ');
        assert_eq!(glyphs[1].codepoint, '.');
        assert_eq!(glyphs[2].codepoint, 'A');
        assert_eq!(glyphs[3].codepoint, 'B');
    }

    #[test]
    fn test_parse_ramp_missing() {
        let ramp = [' ', 'Z'];
        let result = parse_unscii_hex_ramp(TEST_HEX, &ramp);
        assert!(result.is_err());
    }

    #[test]
    fn test_16x16_glyph() {
        let hex_16x16 = "00041:".to_string() + &"FF".repeat(32);
        let glyphs = parse_unscii_hex(&hex_16x16).unwrap();
        let a = glyphs.get(&'A').unwrap();
        assert_eq!(a.width, 16);
        assert_eq!(a.height, 16);
        assert_eq!(a.data.len(), 32);
    }
}