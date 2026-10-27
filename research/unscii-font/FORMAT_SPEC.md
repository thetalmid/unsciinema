# unscii Font Format Specification

## Overview
The unscii font format uses the **Unifont HEX format** — a text-based bitmap font format where each line represents one glyph.

## Format Structure
```
CODEPOINT:HEXBITMAP
```

- **CODEPOINT**: 5-digit hexadecimal Unicode codepoint (e.g., `00041` = 'A')
- **HEXBITMAP**: Hexadecimal string representing the bitmap
- **Glyph dimensions** are determined by the length of the hex string:
  - 16 hex chars = 8×8 pixels (64 bits)
  - 32 hex chars = 8×16 pixels (128 bits)
  - 64 hex chars = 16×16 pixels (256 bits)

## Bit Order
- Bits are ordered **left-to-right, top-to-bottom**
- Each row is 8 bits (1 byte) for 8×8 and 8×16; 16 bits (2 bytes) for 16×16
- MSB (most significant bit) = leftmost pixel
- Example: `00041:00007C12127C1010` (8×8 'A')

## Available Variants (from viznut/unscii fontfiles/)
| File | Size | Style |
|------|------|-------|
| unscii-8.hex | 8×8 | Standard |
| unscii-8-alt.hex | 8×8 | Alternative |
| unscii-8-fantasy.hex | 8×8 | Fantasy |
| unscii-8-mcr.hex | 8×8 | MCR |
| unscii-8-tall.hex | 8×8 | Tall |
| unscii-8-thin.hex | 8×8 | Thin |
| unscii-16.hex | 8×16 | Standard |
| unscii-16-full.hex | 8×16 | Full (with Unifont fallback) |

## Parsing in Rust/WASM
```rust
// Parse .hex format
fn parse_unscii_hex(content: &str) -> HashMap<char, Glyph> {
    content.lines()
        .filter_map(|line| {
            let (cp, bitmap) = line.split_once(':')?;
            let codepoint = u32::from_str_radix(cp, 16).ok()?;
            let ch = char::from_u32(codepoint)?;
            let bytes = hex::decode(bitmap).ok()?;
            // Determine dimensions from byte count
            let (w, h) = match bytes.len() {
                8 => (8, 8),
                16 => (8, 16),
                32 => (16, 16),
                _ => return None,
            };
            Some((ch, Glyph { width: w, height: h, data: bytes }))
        })
        .collect()
}
```

## License
- **Public Domain / CC-0** for most files
- **GPL** for files derived from Unifont (unifont.hex, unscii-16-full.*)
- Source: https://github.com/viznut/unscii

## Download URLs
- https://raw.githubusercontent.com/viznut/unscii/main/fontfiles/unscii-8.hex
- https://raw.githubusercontent.com/viznut/unscii/main/fontfiles/unscii-16.hex
- https://raw.githubusercontent.com/viznut/unscii/main/fontfiles/unscii-8-alt.hex
- etc.