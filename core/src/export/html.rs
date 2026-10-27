//! HTML export with styling and embedded font

use crate::convert::mapper::glyph_to_char;
use crate::export::fonts::{get_font_woff_base64, get_font_woff_format, get_font_family_name};

fn escape_html(s: &str) -> String {
    s.replace("&", "&")
        .replace("<", "<")
        .replace(">", ">")
        .replace("\"", "&quot;")
        .replace('\'', "&#39;")
}

fn escape_css(s: &str) -> String {
    s.replace("\\", "\\\\")
        .replace("\"", "\\\"")
        .replace('\'', "\\'")
        .replace("
", "")
}

const CSS_TEMPLATE: &str = "body { background: {bg}; margin: 0; padding: 20px; font-family: {font}; }
pre { 
    color: {fg}; 
    background: {bg}; 
    font-size: {fs}px; 
    line-height: {lh}; 
    white-space: pre; 
    overflow-x: auto;
    display: inline-block;
    padding: 10px;
}
@font-face {
    font-family: \"{font}\";
    src: url(\"data:font/{format};base64,{font_data}\") format(\"{format}\");
    font-weight: normal;
    font-style: normal;
}";

pub fn export_html(
    grid: &[u16],
    cols: u32,
    rows: u32,
    fg_color: &str,
    bg_color: &str,
    font_family: &str,
    line_height: f32,
    font_size: u32,
    font_variant: crate::font::variants::FontVariant,
) -> String {
    let mut html = String::new();
    
    let safe_fg = escape_css(fg_color);
    let safe_bg = escape_css(bg_color);
    let _safe_font_family = escape_css(font_family);
    let font_name = get_font_family_name(font_variant);
    let safe_font_name = escape_css(font_name);
    let font_base64 = get_font_woff_base64(font_variant);
    let font_format = get_font_woff_format(font_variant);
    
    let css = CSS_TEMPLATE
        .replace("{bg}", &safe_bg)
        .replace("{fg}", &safe_fg)
        .replace("{font}", &safe_font_name)
        .replace("{fs}", &font_size.to_string())
        .replace("{lh}", &line_height.to_string())
        .replace("{format}", font_format)
        .replace("{font_data}", font_base64);
    
    html.push_str("<!DOCTYPE html>
<html>
<head>
<meta charset=\"UTF-8\">
<title>Unscii ASCII Art - ");
    html.push_str(&escape_html(&safe_font_name));
    html.push_str("</title>
<style>
");
    html.push_str(&css);
    html.push_str("
</style>
</head>
<body><pre>");
    
    for row in 0..rows {
        for col in 0..cols {
            let idx = grid[(row * cols + col) as usize] as usize;
            let ch = glyph_to_char(idx as u16);
            let escaped = escape_html(&ch.to_string());
            html.push_str(&escaped);
        }
        if row < rows - 1 {
            html.push('\n');
        }
    }
    
    html.push_str("</pre></body></html>");
    html
}
