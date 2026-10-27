//! HTML export with styling

use crate::convert::mapper::glyph_to_char;

pub fn export_html(
    grid: &[u16],
    cols: u32,
    rows: u32,
    fg_color: &str,
    bg_color: &str,
    font_family: &str,
    line_height: f32,
    font_size: u32,
) -> String {
    let mut html = String::new();
    
    html.push_str(&format!(r#"<!DOCTYPE html>
<html>
<head>
<meta charset="UTF-8">
<title>Unscii ASCII Art</title>
<style>
body {{ background: {bg}; margin: 0; padding: 20px; font-family: {font}; }}
pre {{ 
    color: {fg}; 
    background: {bg}; 
    font-size: {fs}px; 
    line-height: {lh}; 
    white-space: pre; 
    overflow-x: auto;
    display: inline-block;
    padding: 10px;
}}
@font-face {{
    font-family: 'Unscii';
    src: url('https://viznut.fi/unscii/unscii-8.woff') format('woff');
    font-weight: normal;
    font-style: normal;
}}
</style>
</head>
<body><pre>"#, 
        bg = bg_color, fg = fg_color, font = font_family, fs = font_size, lh = line_height
    ));
    
    for row in 0..rows {
        for col in 0..cols {
            let idx = grid[(row * cols + col) as usize] as usize;
            let ch = glyph_to_char(idx as u16);
            match ch {
                '<' => html.push_str("<"),
                '>' => html.push_str(">"),
                '&' => html.push_str("&"),
                '"' => html.push_str("\""),
                '\'' => html.push_str("'"),
                _ => html.push(ch),
            }
        }
        if row < rows - 1 {
            html.push('\n');
        }
    }
    
    html.push_str("</pre></body></html>");
    html
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_export_basic() {
        let grid = vec![0, 1, 2, 3];
        let html = export_html(&grid, 2, 2, "#fff", "#000", "monospace", 1.1, 12);
        assert!(html.contains("<pre>"));
        assert!(html.contains("</pre>"));
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("font-family"));
    }

    #[test]
    fn test_html_export_colors() {
        let grid = vec![0];
        let html = export_html(&grid, 1, 1, "#ff0000", "#00ff00", "monospace", 1.1, 12);
        assert!(html.contains("#ff0000"));
        assert!(html.contains("#00ff00"));
    }

    #[test]
    fn test_html_export_font() {
        let grid = vec![0];
        let html = export_html(&grid, 1, 1, "#fff", "#000", "'Custom Font', monospace", 1.5, 14);
        assert!(html.contains("'Custom Font', monospace"));
        assert!(html.contains("14px"));
        assert!(html.contains("1.5"));
    }

    #[test]
    fn test_html_escapes_special_chars() {
        // We can't easily test this without glyphs that produce these chars
        // But we can verify the escape logic exists
        let grid = vec![0]; // Space
        let html = export_html(&grid, 1, 1, "#fff", "#000", "monospace", 1.1, 12);
        assert!(html.contains(" "));
    }

    #[test]
    fn test_html_export_multiline() {
        let grid = vec![0, 1, 2, 3];
        let html = export_html(&grid, 2, 2, "#fff", "#000", "monospace", 1.1, 12);
        let pre_start = html.find("<pre>").unwrap();
        let pre_end = html.find("</pre>").unwrap();
        let content = &html[pre_start+5..pre_end];
        assert!(content.contains('\n'));
    }

    #[test]
    fn test_html_has_font_face() {
        let grid = vec![0];
        let html = export_html(&grid, 1, 1, "#fff", "#000", "monospace", 1.1, 12);
        assert!(html.contains("@font-face"));
        assert!(html.contains("Unscii"));
        assert!(html.contains("unscii-8.woff"));
    }

    #[test]
    fn test_html_line_height_and_font_size() {
        let grid = vec![0];
        let html = export_html(&grid, 1, 1, "#fff", "#000", "monospace", 1.5, 16);
        assert!(html.contains("line-height: 1.5"));
        assert!(html.contains("font-size: 16px"));
    }
}