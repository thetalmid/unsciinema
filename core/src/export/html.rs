//! Standalone HTML export with an embedded real WOFF font and per-cell colors.
//!
//! Safety properties:
//! - every grid character passes through full HTML escaping;
//! - colors are generated from validated u8 arrays and re-checked by the
//!   strict `css_color` validator before touching markup;
//! - no scripts, no event handlers, no external requests (CSP meta: no
//!   script sources at all; fonts only from `data:`);
//! - the document discloses that browser font rasterization may differ from
//!   the bitmap preview.

use crate::error::{CoreError, Result};
use crate::font::VariantFont;
use crate::limits::{checked_cells, MAX_HTML_COLORED_CELLS};

/// HTML-escape (salvage helper, extended with `"` and `'` already present).
fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

/// Strict validator for the `#rrggbb` / `#rrggbbaa` values we generate.
fn validate_css_color(s: &str) -> Result<&str> {
    let body = s.strip_prefix('#').unwrap_or("");
    let ok = (body.len() == 6 || body.len() == 8) && body.chars().all(|c| c.is_ascii_hexdigit());
    if ok {
        Ok(s)
    } else {
        Err(CoreError::export(format!("generated invalid color {s:?}")))
    }
}

/// Serialize a straight RGBA cell color to validated CSS hex.
fn css_color(rgba: [u8; 4]) -> Result<String> {
    let s = if rgba[3] == 255 {
        format!("#{:02x}{:02x}{:02x}", rgba[0], rgba[1], rgba[2])
    } else {
        format!("#{:02x}{:02x}{:02x}{:02x}", rgba[0], rgba[1], rgba[2], rgba[3])
    };
    validate_css_color(&s)?;
    Ok(s)
}

/// Minimal base64 (no external dependency).
fn base64(data: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32;
        out.push(T[(n >> 18) as usize & 63] as char);
        out.push(T[(n >> 12) as usize & 63] as char);
        if chunk.len() > 1 {
            out.push(T[(n >> 6) as usize & 63] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(T[n as usize & 63] as char);
        } else {
            out.push('=');
        }
    }
    out
}

/// Family name: derived from the variant name (already `[a-z0-9-]`, filtered
/// again defensively so nothing else can reach the style block).
fn family_name(font: &VariantFont) -> String {
    font.variant
        .name()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .collect::<String>()
        + "UnsciiExport"
}

pub struct HtmlExportOptions {
    /// Emit per-cell color spans (color/palette modes).
    pub colored: bool,
}

pub fn export_html(
    grid: &[u16],
    fg: &[u8],
    bg: &[u8],
    cols: u32,
    rows: u32,
    font: &VariantFont,
    woff: &[u8],
    opts: &HtmlExportOptions,
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
    if opts.colored && cells as u64 > MAX_HTML_COLORED_CELLS {
        return Err(CoreError::limit(format!(
            "grid has {cells} cells; colored HTML export is limited to {MAX_HTML_COLORED_CELLS} (use mono colors or raster exports for larger grids)"
        )));
    }
    if woff.len() < 4 || &woff[0..4] != b"wOFF" {
        return Err(CoreError::export(
            "embedded font is not a WOFF file (refusing to pretend otherwise)",
        ));
    }

    let family = family_name(font);
    let default_fg = css_color([fg[0], fg[1], fg[2], fg[3]])?;
    let default_bg = css_color([bg[0], bg[1], bg[2], bg[3]])?;

    let mut body = String::with_capacity(cells * 24 + 256);
    for row in 0..rows {
        if row > 0 {
            body.push('\n');
        }
        if opts.colored {
            for col in 0..cols {
                let i = (row * cols + col) as usize;
                let f = css_color([fg[i * 4], fg[i * 4 + 1], fg[i * 4 + 2], fg[i * 4 + 3]])?;
                let b = css_color([bg[i * 4], bg[i * 4 + 1], bg[i * 4 + 2], bg[i * 4 + 3]])?;
                body.push_str("<span style=\"color:");
                body.push_str(&f);
                body.push_str(";background:");
                body.push_str(&b);
                body.push_str("\">");
                body.push_str(&escape_html(&font.glyph_to_char(grid[i]).to_string()));
                body.push_str("</span>");
            }
        } else {
            for col in 0..cols {
                let i = (row * cols + col) as usize;
                body.push_str(&escape_html(&font.glyph_to_char(grid[i]).to_string()));
            }
        }
    }

    let b64 = base64(woff);
    let doc = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline'; font-src data:; img-src data:">
<title>Unscii conversion ({family})</title>
<!-- Generated by Unsciinema. Self-contained: no external requests, no scripts.
     The embedded {family} font is the vectorized WOFF from Unscii by viznut
     (Public Domain). Note: browsers rasterize the vectorized font, so glyph
     rendering may differ slightly from the pixel-exact bitmap preview. -->
<style>
@font-face {{
  font-family: '{family}';
  src: url(data:font/woff;base64,{b64}) format('woff');
  font-display: block;
}}
html, body {{ margin: 0; padding: 0; background: #0d0d0d; }}
pre.unscii {{
  font-family: '{family}', ui-monospace, monospace;
  font-size: 14px;
  line-height: 1;
  white-space: pre;
  letter-spacing: 0;
  margin: 0;
  padding: 12px;
  color: {default_fg};
  background: {default_bg};
}}
pre.unscii span {{ text-decoration: none; }}
</style>
</head>
<body>
<pre class="unscii">{body}</pre>
</body>
</html>
"#
    );
    Ok(doc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::{FontVariant, VariantFont};

    #[test]
    fn escapes_hostile_content() {
        let font = VariantFont::load(FontVariant::Unscii8).unwrap();
        // All cells = the densest glyph (never '<' etc, but the escaper is
        // still exercised on whatever chars the ramp provides).
        let cells = 4usize;
        let grid = vec![15u16; cells];
        let fg = vec![255u8; cells * 4];
        let bg = vec![0u8; cells * 4];
        let woff = b"wOFF-fake-for-test".to_vec();
        let html = export_html(
            &grid,
            &fg,
            &bg,
            2,
            2,
            &font,
            &woff,
            &HtmlExportOptions { colored: true },
        )
        .unwrap();
        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.contains("data:font/woff;base64,"));
        assert!(html.contains("Content-Security-Policy"));
        assert!(!html.contains("<script"));
        // Direct escaper check with hostile input (as if a ramp contained
        // markup-relevant characters): nothing can break out of the text node.
        assert_eq!(
            escape_html("<script>alert(1)</script>&"),
            "&lt;script&gt;alert(1)&lt;/script&gt;&amp;"
        );
        assert_eq!(escape_html("a\"b'c"), "a&quot;b&#39;c");
        assert!(html.contains("data:font/woff;base64,"));
        assert!(html.contains("Content-Security-Policy"));
        assert!(!html.contains("<script"));
        // Family name comes from the variant table only.
        assert!(html.contains("unscii-8UnsciiExport"));
    }

    #[test]
    fn rejects_non_woff_and_bad_lengths() {
        let font = VariantFont::load(FontVariant::Unscii8).unwrap();
        let grid = vec![0u16; 4];
        let fg = vec![255u8; 16];
        let bg = vec![0u8; 16];
        let err = export_html(
            &grid,
            &fg,
            &bg,
            2,
            2,
            &font,
            b"NOTAWOFF",
            &HtmlExportOptions { colored: false },
        );
        assert!(err.is_err());
        assert!(export_html(
            &grid,
            &[0u8; 3],
            &bg,
            2,
            2,
            &font,
            b"wOFF",
            &HtmlExportOptions { colored: false },
        )
        .is_err());
    }

    #[test]
    fn colored_size_cap() {
        let font = VariantFont::load(FontVariant::Unscii8).unwrap();
        // Build a grid over the cap: 501 x 500 = 250500 > 250000.
        let cols = 501u32;
        let rows = 500u32;
        let cells = (cols * rows) as usize;
        let grid = vec![0u16; cells];
        let fg = vec![255u8; cells * 4];
        let bg = vec![0u8; cells * 4];
        let res = export_html(
            &grid,
            &fg,
            &bg,
            cols,
            rows,
            &font,
            b"wOFF",
            &HtmlExportOptions { colored: true },
        );
        assert!(res.is_err());
        // Mono mode is allowed at the same size (plain text only).
        let res = export_html(
            &grid,
            &fg,
            &bg,
            cols,
            rows,
            &font,
            b"wOFF",
            &HtmlExportOptions { colored: false },
        );
        assert!(res.is_ok());
    }

    #[test]
    fn css_color_strict() {
        assert!(validate_css_color("#000000").is_ok());
        assert!(validate_css_color("#000000ff").is_ok());
        assert!(validate_css_color("#000").is_err());
        assert!(validate_css_color("#000000gg").is_err());
        assert!(validate_css_color("red; background: url(x)").is_err());
        assert_eq!(css_color([255, 0, 128, 255]).unwrap(), "#ff0080");
        assert_eq!(css_color([255, 0, 128, 128]).unwrap(), "#ff008080");
    }
}
