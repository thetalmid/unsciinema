//! Image to ASCII grid conversion

pub mod mapper;
pub mod grid;

use crate::convert::mapper::map_luma_to_glyphs;
pub use crate::convert::grid::AsciiGrid;
use crate::convert::grid::AsciiGrid as _;
use crate::font::atlas::FontAtlas;
use crate::font::variants::FontVariant;
use crate::dither::apply_dither;
use crate::image::ImageProcessor;
use wasm_bindgen::prelude::*;

/// Main conversion function
pub fn convert_image(
    processor: &mut ImageProcessor,
    params: &ConvertParams,
    font_atlas: &FontAtlas,
) -> Result<AsciiGrid, JsValue> {
    // Apply preprocessing
    processor.apply_preprocessing(params.contrast, params.gamma);
    
    // Resize if target width specified
    if params.target_width > 0 {
        processor.resize(params.target_width);
    }
    
    // Get luma buffer
    let luma = processor.luma();
    let width = processor.width();
    let height = processor.height();
    
    // Apply dithering (output to separate buffer)
    let mut dithered = vec![0u8; (width * height) as usize];
    apply_dither(
        &luma,
        &mut dithered,
        width,
        height,
        params.dither_algorithm,
        params.dither_strength,
        params.threshold,
    );
    
    // Map to glyphs using dithered buffer (now averages cell)
    let cell_size = params.cell_size.max(1.0);
    let mut grid = map_luma_to_glyphs(&dithered, width, height, cell_size, params.threshold)?;
    
    // Render to RGBA for preview (using font atlas)
    let rgba = render_grid_to_rgba(&grid, font_atlas, cell_size, processor.rgba().as_slice(), width, height, params.color_mode, &params.palette, params.palette_size as usize)?;
    
    grid.set_rgba(rgba);
    Ok(grid)
}

fn render_grid_to_rgba(
    grid: &AsciiGrid,
    font_atlas: &FontAtlas,
    cell_size: f32,
    source_rgba: &[u8],
    src_width: u32,
    src_height: u32,
    color_mode: u32,
    palette: &[u32],
    palette_size: usize,
) -> Result<Vec<u8>, JsValue> {
    let cols = grid.cols();
    let rows = grid.rows();
    let glyph_w = font_atlas.glyph_width;
    let glyph_h = font_atlas.glyph_height;
    let out_width = cols * glyph_w;
    let out_height = rows * glyph_h;
    let mut out = vec![0u8; (out_width * out_height * 4) as usize];
    
    // Limit palette to palette_size
    let palette = &palette[..palette_size.min(palette.len())];
    
    for row in 0..rows {
        for col in 0..cols {
            let glyph_idx = grid.get(col, row) as usize;
            
            // Source cell center for color sampling
            let src_cx = ((col as f32 + 0.5) * cell_size) as u32;
            let src_cy = ((row as f32 + 0.5) * cell_size) as u32;
            let src_cx = src_cx.min(src_width - 1);
            let src_cy = src_cy.min(src_height - 1);
            let src_idx = ((src_cy * src_width + src_cx) * 4) as usize;
            let src_color = &source_rgba[src_idx..src_idx + 4];
            
            // Compute luminance from source color (BT.709)
            let lum = (0.2126 * src_color[0] as f32 + 0.7152 * src_color[1] as f32 + 0.0722 * src_color[2] as f32) as u8;
            
            // Render glyph
            for gy in 0..glyph_h {
                for gx in 0..glyph_w {
                    let atlas_idx = ((gy as usize * font_atlas.width as usize + glyph_idx as usize * glyph_w as usize + gx as usize) * 4) as usize;
                    let ink = font_atlas.data[atlas_idx] > 128;
                    
                    let out_x = col * glyph_w + gx;
                    let out_y = row * glyph_h + gy;
                    let out_idx = ((out_y * out_width + out_x) * 4) as usize;
                    
                    if ink {
                        match color_mode {
                            0 => { // Mono - use luminance for grayscale
                                out[out_idx] = lum;
                                out[out_idx + 1] = lum;
                                out[out_idx + 2] = lum;
                                out[out_idx + 3] = 255;
                            }
                            1 => { // Color - use source color
                                out[out_idx] = src_color[0];
                                out[out_idx + 1] = src_color[1];
                                out[out_idx + 2] = src_color[2];
                                out[out_idx + 3] = src_color[3];
                            }
                            2 => { // Palette - map luminance to palette
                                if palette.len() >= 2 {
                                    let normalized = lum as f32 / 255.0;
                                    let palette_idx = (normalized * (palette.len() - 1) as f32).round() as usize;
                                    let palette_idx = palette_idx.min(palette.len() - 1);
                                    let p = palette[palette_idx];
                                    out[out_idx] = (p >> 24) as u8;
                                    out[out_idx + 1] = (p >> 16) as u8;
                                    out[out_idx + 2] = (p >> 8) as u8;
                                    out[out_idx + 3] = p as u8;
                                } else {
                                    // Fallback to mono
                                    out[out_idx] = lum;
                                    out[out_idx + 1] = lum;
                                    out[out_idx + 2] = lum;
                                    out[out_idx + 3] = 255;
                                }
                            }
                            _ => { out[out_idx..out_idx+4].copy_from_slice(src_color); }
                        }
                    } else {
                        // Background - dark
                        out[out_idx] = 0;
                        out[out_idx + 1] = 0;
                        out[out_idx + 2] = 0;
                        out[out_idx + 3] = 255;
                    }
                }
            }
        }
    }
    
    Ok(out)
}

#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct ConvertParams {
    pub dither_algorithm: u32,
    pub dither_strength: f32,
    pub threshold: f32,
    pub font_variant: u32,
    pub cell_size: f32,
    pub color_mode: u32,
    #[wasm_bindgen(getter_with_clone)]
    pub palette: Vec<u32>,
    pub palette_size: u32,
    pub alpha_mode: u32,
    pub contrast: f32,
    pub gamma: f32,
    pub target_width: u32,
    pub crt_scanlines: bool,
    pub crt_curvature: f32,
    pub crt_bloom: f32,
    pub crt_vignette: f32,
}

impl Default for ConvertParams {
    fn default() -> Self {
        Self {
            dither_algorithm: 0,
            dither_strength: 1.0,
            threshold: 0.5,
            font_variant: 0,
            cell_size: 8.0,
            color_mode: 0,
            palette: vec![0x000000FF, 0xFFFFFFFF],
            palette_size: 2,
            alpha_mode: 0,
            contrast: 1.0,
            gamma: 1.0,
            target_width: 0,
            crt_scanlines: false,
            crt_curvature: 0.0,
            crt_bloom: 0.0,
            crt_vignette: 0.0,
        }
    }
}

#[wasm_bindgen]
impl ConvertParams {
    #[wasm_bindgen(constructor)]
    pub fn new() -> ConvertParams {
        ConvertParams::default()
    }
}