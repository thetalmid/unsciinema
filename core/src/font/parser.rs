//! unscii `.hex` format parsing.
//!
//! Format: `CODEPOINT:HEXBITMAP`
//! - `CODEPOINT`: hex Unicode codepoint (5 digits in unscii files, e.g. `00041` = 'A')
//! - `HEXBITMAP`: hex string whose length determines dimensions:
//!   - 16 hex chars (8 bytes)  -> 8x8
//!   - 32 hex chars (16 bytes) -> 8x16   (double-height glyphs)
//!   - 64 hex chars (32 bytes) -> 16x16  (DBLW wide glyphs in unscii-16.hex)
//!
//! Bit order: MSB first, left-to-right, top-to-bottom.
//!
//! Note: assemble.pl writes the U+0000 line of *every* file as 32 zero hex
//! chars regardless of file, so a lone 16-byte all-zero record in an 8x8 file
//! is an upstream quirk. Ramp glyphs never include U+0000; atlas building
//! filters glyphs by the variant's expected dimensions, which sidesteps it.

use crate::error::{CoreError, Result};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Glyph {
    pub codepoint: char,
    pub width: u32,
    pub height: u32,
    /// Raw bitmap bytes (1 bit per pixel, MSB first, row-major).
    pub data: Vec<u8>,
}

impl Glyph {
    /// Pixel at (x, y): true when the bit is set (ink).
    pub fn get_pixel(&self, x: u32, y: u32) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }
        let bytes_per_row = self.width.div_ceil(8);
        let byte_index = (y * bytes_per_row + x / 8) as usize;
        let bit_index = 7 - (x % 8);
        match self.data.get(byte_index) {
            Some(b) => (b >> bit_index) & 1 == 1,
            None => false,
        }
    }

    /// Number of ink bits / total bits, in [0, 1].
    pub fn ink_density(&self) -> f64 {
        let total = (self.width as u64) * (self.height as u64);
        if total == 0 {
            return 0.0;
        }
        let mut ink = 0u64;
        for y in 0..self.height {
            for x in 0..self.width {
                if self.get_pixel(x, y) {
                    ink += 1;
                }
            }
        }
        ink as f64 / total as f64
    }
}

/// Parse unscii `.hex` content into a codepoint map.
pub fn parse_unscii_hex(content: &str) -> Result<HashMap<char, Glyph>> {
    let mut glyphs = HashMap::new();

    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (cp_str, hex_str) = line.split_once(':').ok_or_else(|| {
            CoreError::font(format!("line {}: missing ':'", line_num + 1))
        })?;
        let codepoint = u32::from_str_radix(cp_str.trim(), 16).map_err(|_| {
            CoreError::font(format!("line {}: bad codepoint '{}'", line_num + 1, cp_str))
        })?;
        let ch = char::from_u32(codepoint).ok_or_else(|| {
            CoreError::font(format!("line {}: invalid unicode {codepoint}", line_num + 1))
        })?;
        let bytes = hex::decode(hex_str.trim())
            .map_err(|e| CoreError::font(format!("line {}: {e}", line_num + 1)))?;

        let (width, height) = match bytes.len() {
            8 => (8, 8),
            16 => (8, 16),
            32 => (16, 16),
            n => {
                return Err(CoreError::font(format!(
                    "line {}: unsupported glyph size {n} bytes",
                    line_num + 1
                )))
            }
        };
        glyphs.insert(
            ch,
            Glyph {
                codepoint: ch,
                width,
                height,
                data: bytes,
            },
        );
    }
    Ok(glyphs)
}

/// Parse and keep only the requested characters, filtered to `expected`
/// dimensions when provided (skips upstream anomalies like the U+0000 line).
pub fn parse_glyphs_for(
    content: &str,
    chars: &[char],
    expected: Option<(u32, u32)>,
) -> Result<HashMap<char, Glyph>> {
    let all = parse_unscii_hex(content)?;
    let mut out = HashMap::new();
    for &ch in chars {
        if let Some(g) = all.get(&ch) {
            let dims_ok = match expected {
                Some((w, h)) => g.width == w && g.height == h,
                None => true,
            };
            if dims_ok {
                out.insert(ch, g.clone());
            }
        }
    }
    Ok(out)
}
