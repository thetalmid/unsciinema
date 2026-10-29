//! The conversion pipeline: immutable source -> preprocessing -> cell
//! sampling -> dither/quantize -> per-cell colors + glyph grid.
//!
//! Cell geometry (documented choice for aspect preservation):
//!   `box_h = cell_size` (source pixels consumed per cell row),
//!   `box_w = cell_size * (glyph_w / glyph_h)` (sampling box matches glyph
//!   aspect, so `cols*glyph_w / rows*glyph_h` tracks the source aspect even
//!   for the 8x16 tall variants).
//! `cols = ceil(W / box_w)`, `rows = ceil(H / box_h)`; fractional boundaries
//! are computed in f64 and cells at the right/bottom edge are partial (they
//! average only the pixels that exist). If `box` < 1px a cell falls back to
//! exactly one clamped pixel — deterministic, gap- and OOB-free.

use crate::dither::{palette, DitherAlgorithm};
use crate::error::{CoreError, Result};
use crate::font::VariantFont;
use crate::image::loader::DecodedImage;
use crate::image::luminance::{bt709_luma, stage_rgb_luma, AlphaMode};
use crate::image::preprocessing::{apply_contrast, apply_gamma, resize_target, validate_contrast, validate_gamma};
use crate::limits::{check_f32, checked_cells, MAX_SOURCE_DIM};

pub mod grid;

use self::grid::AsciiGrid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    Mono = 0,
    Color = 1,
    Palette = 2,
}

impl ColorMode {
    pub fn from_u32(v: u32) -> Option<ColorMode> {
        match v {
            0 => Some(ColorMode::Mono),
            1 => Some(ColorMode::Color),
            2 => Some(ColorMode::Palette),
            _ => None,
        }
    }
}

/// Fully validated settings for one conversion.
#[derive(Debug, Clone)]
pub struct ConvertSettings {
    pub dither_algorithm: DitherAlgorithm,
    pub dither_strength: f32,
    pub threshold: f32,
    pub cell_size: f32,
    pub color_mode: ColorMode,
    pub alpha_mode: AlphaMode,
    pub contrast: f32,
    pub gamma: f32,
    pub target_width: u32,
    /// Background color, 0xRRGGBBAA.
    pub background: u32,
    /// Palette colors (only used in Palette mode), 0xRRGGBBAA.
    pub palette: Vec<u32>,
}

/// Result of one frame conversion (pre-WASM-wrapper form).
#[derive(Debug, Clone)]
pub struct ConversionData {
    pub grid: AsciiGrid,
    /// cols*rows*4, straight RGBA.
    pub cell_fg: Vec<u8>,
    /// cols*rows*4, straight RGBA.
    pub cell_bg: Vec<u8>,
    pub glyph_width: u32,
    pub glyph_height: u32,
    /// The (possibly resized) source dimensions actually sampled.
    /// (Diagnostics surfaced by tests/benchmarks; not part of the WASM
    /// ConversionResult contract.)
    #[allow(dead_code)]
    pub sampled_width: u32,
    #[allow(dead_code)]
    pub sampled_height: u32,
}

fn unpack(packed: u32) -> [u8; 4] {
    [
        (packed >> 24) as u8,
        (packed >> 16) as u8,
        (packed >> 8) as u8,
        packed as u8,
    ]
}

#[inline]
fn clamp_round(v: f32) -> u8 {
    v.round().clamp(0.0, 255.0) as u8
}

/// Run the full still-frame conversion.
pub fn convert_frame(
    frame: &DecodedImage,
    font: &VariantFont,
    s: &ConvertSettings,
) -> Result<ConversionData> {
    // --- validate everything at this boundary -----------------------------
    let cell_size = check_f32("cell_size", s.cell_size, 1.0, 64.0)?;
    validate_contrast(s.contrast)?;
    validate_gamma(s.gamma)?;
    if s.target_width > MAX_SOURCE_DIM {
        return Err(CoreError::limit(format!(
            "target_width {} exceeds max {MAX_SOURCE_DIM}",
            s.target_width
        )));
    }
    if s.color_mode == ColorMode::Palette {
        palette::validate_palette(&s.palette)?;
    }
    let bg = unpack(s.background);

    // --- 1. work on a copy: resize -> contrast -> gamma -------------------
    let (sw, sh) = (frame.width, frame.height);
    if sw == 0 || sh == 0 {
        return Err(CoreError::invalid("source has zero dimension"));
    }
    let rgba_img = image::RgbaImage::from_raw(sw, sh, frame.rgba.clone())
        .ok_or_else(|| CoreError::state("source buffer size mismatch"))?;
    let resized = resize_target(&rgba_img, s.target_width)?;
    let resized_dims = resized.as_ref().map(|img| img.dimensions());
    let mut work: Vec<u8> = match resized {
        Some(img) => img.into_raw(),
        None => frame.rgba.clone(),
    };
    apply_contrast(&mut work, s.contrast);
    apply_gamma(&mut work, s.gamma);

    // --- 2. alpha stage: composite over background when requested ---------
    let alpha_mode = s.alpha_mode;
    if alpha_mode == AlphaMode::Blend && bg[3] > 0 {
        let (br, bgc, bb, ba) = (bg[0] as f32, bg[1] as f32, bg[2] as f32, bg[3] as f32 / 255.0);
        for px in work.chunks_exact_mut(4) {
            let sa = px[3] as f32 / 255.0;
            let (r, g, b, a) = crate::image::luminance::over_straight(
                px[0] as f32,
                px[1] as f32,
                px[2] as f32,
                sa,
                br,
                bgc,
                bb,
                ba,
            );
            px[0] = clamp_round(r);
            px[1] = clamp_round(g);
            px[2] = clamp_round(b);
            px[3] = clamp_round(a * 255.0);
        }
    }

    let (w, h) = resized_dims.unwrap_or((sw, sh));
    let work_pixels = (w as usize) * (h as usize);
    if work.len() != work_pixels * 4 {
        return Err(CoreError::state("processed buffer size mismatch"));
    }

    // --- 3. cell geometry --------------------------------------------------
    let gw = font.atlas.glyph_width;
    let gh = font.atlas.glyph_height;
    let box_w = cell_size as f64 * (gw as f64 / gh as f64);
    let box_h = cell_size as f64;
    if !box_w.is_finite() || box_w <= 0.0 || !box_h.is_finite() || box_h <= 0.0 {
        return Err(CoreError::invalid("cell size produced an invalid sampling box"));
    }
    let cols_f = (w as f64 / box_w).ceil();
    let rows_f = (h as f64 / box_h).ceil();
    let cols = cols_f.max(1.0) as u32;
    let rows = rows_f.max(1.0) as u32;
    let cells = checked_cells(cols, rows)?;

    // --- 4. per-cell stats + luma grid ------------------------------------
    let bg_rgb_f = [bg[0] as f32, bg[1] as f32, bg[2] as f32];
    let mut luma_grid = vec![0f32; cells];
    let mut mean_rgb = vec![[0.0f32; 3]; cells];
    let mut mean_a = vec![0f32; cells];
    let mut max_a = vec![0f32; cells];

    for row in 0..rows {
        for col in 0..cols {
            let idx = (row * cols + col) as usize;
            let x0 = ((col as f64) * box_w).floor();
            let x1 = (((col as f64) + 1.0) * box_w).floor();
            let y0 = ((row as f64) * box_h).floor();
            let y1 = (((row as f64) + 1.0) * box_h).floor();

            let cx0 = x0.max(0.0).min(w as f64 - 1.0) as u32;
            let mut cx1 = x1.min(w as f64) as u32;
            let cy0 = y0.max(0.0).min(h as f64 - 1.0) as u32;
            let mut cy1 = y1.min(h as f64) as u32;
            // Fractional cells narrower than a pixel: sample exactly one
            // clamped pixel so every cell is defined (no gaps, no OOB).
            if cx1 <= cx0 {
                cx1 = cx0 + 1;
            }
            if cy1 <= cy0 {
                cy1 = cy0 + 1;
            }

            let mut sum_luma = 0f64;
            let mut sum_rgb = [0f64; 3];
            let mut sum_a = 0f64;
            let mut mx_a = 0f64;
            let mut count = 0u32;
            for cy in cy0..cy1 {
                for cx in cx0..cx1 {
                    let p = ((cy * w + cx) * 4) as usize;
                    let r = work[p] as f32;
                    let g = work[p + 1] as f32;
                    let b = work[p + 2] as f32;
                    let a = work[p + 3] as f32 / 255.0;
                    let (sr, sg, sb) = stage_rgb_luma(r, g, b, a, alpha_mode, bg_rgb_f);
                    sum_luma += bt709_luma(sr, sg, sb) as f64;
                    sum_rgb[0] += sr as f64;
                    sum_rgb[1] += sg as f64;
                    sum_rgb[2] += sb as f64;
                    sum_a += a as f64;
                    if a as f64 > mx_a {
                        mx_a = a as f64;
                    }
                    count += 1;
                }
            }
            let n = count.max(1) as f64;
            luma_grid[idx] = (sum_luma / n / 255.0) as f32;
            mean_rgb[idx] = [
                (sum_rgb[0] / n) as f32,
                (sum_rgb[1] / n) as f32,
                (sum_rgb[2] / n) as f32,
            ];
            mean_a[idx] = (sum_a / n) as f32;
            max_a[idx] = mx_a as f32;
        }
    }

    // --- 5. dither/quantize cell luminance -> glyph indices ----------------
    let levels = crate::dither::quantize_grid(
        &luma_grid,
        cols,
        rows,
        s.dither_algorithm,
        s.dither_strength,
        s.threshold,
    )?;
    // Checked write path: AsciiGrid::set validates row/col bounds per cell.
    let mut grid = AsciiGrid::new(cols, rows)?;
    for (i, &lvl) in levels.iter().enumerate() {
        let col = (i as u32) % cols;
        let row = (i as u32) / cols;
        grid.set(col, row, lvl)?;
    }

    // --- 6. per-cell colors ------------------------------------------------
    let mut cell_fg = vec![0u8; cells * 4];
    let mut cell_bg = vec![0u8; cells * 4];
    for idx in 0..cells {
        let mean = mean_rgb[idx];
        let mr = clamp_round(mean[0]);
        let mg = clamp_round(mean[1]);
        let mb = clamp_round(mean[2]);

        // Foreground RGB.
        let (fr, fg_, fb, mut fa) = match s.color_mode {
            ColorMode::Mono => (255u8, 255u8, 255u8, 255u8),
            ColorMode::Color => (mr, mg, mb, 255u8),
            ColorMode::Palette => {
                let i = palette::nearest_index(&s.palette, mr, mg, mb);
                let e = unpack(s.palette[i]);
                (e[0], e[1], e[2], e[3])
            }
        };
        // Foreground alpha rules (see INTEGRATION.md).
        fa = match alpha_mode {
            AlphaMode::Ignore | AlphaMode::Blend => fa,
            AlphaMode::Preserve => {
                let src_max = clamp_round(max_a[idx] * 255.0);
                ((fa as u32 * src_max as u32) / 255) as u8
            }
        };

        // Background RGB.
        let (bgr, bgg, bgb) = match s.color_mode {
            ColorMode::Palette => {
                let i = palette::nearest_index(&s.palette, bg[0], bg[1], bg[2]);
                let e = unpack(s.palette[i]);
                (e[0], e[1], e[2])
            }
            _ => (bg[0], bg[1], bg[2]),
        };
        // Background alpha rules.
        let bga = match alpha_mode {
            AlphaMode::Preserve => clamp_round(mean_a[idx] * bg[3] as f32),
            AlphaMode::Ignore | AlphaMode::Blend => bg[3],
        };

        let o = idx * 4;
        cell_fg[o] = fr;
        cell_fg[o + 1] = fg_;
        cell_fg[o + 2] = fb;
        cell_fg[o + 3] = fa;
        cell_bg[o] = bgr;
        cell_bg[o + 1] = bgg;
        cell_bg[o + 2] = bgb;
        cell_bg[o + 3] = bga;
    }

    Ok(ConversionData {
        grid,
        cell_fg,
        cell_bg,
        glyph_width: gw,
        glyph_height: gh,
        sampled_width: w,
        sampled_height: h,
    })
}

/// Core CPU reference raster over raw cell arrays (straight RGBA, top-left
/// origin). This single implementation backs both `rasterize_ascii` (native
/// tests) and the WASM `UnsciiConverter::raster` — the WebGL2 glyph pass and
/// the Canvas2D fallback must match it when CRT effects are off. Blending with
/// the original image happens in the renderer, not here.
pub fn raster_cells(
    indices: &[u16],
    fg: &[u8],
    bg: &[u8],
    cols: u32,
    rows: u32,
    atlas: &crate::font::atlas::FontAtlas,
) -> Result<Vec<u8>> {
    let cells = checked_cells(cols, rows)?;
    if indices.len() != cells {
        return Err(CoreError::invalid(format!(
            "indices has {} entries, expected {cells}",
            indices.len()
        )));
    }
    if fg.len() != cells * 4 || bg.len() != cells * 4 {
        return Err(CoreError::invalid("cell color arrays must be cols*rows*4"));
    }
    let gw = atlas.glyph_width;
    let gh = atlas.glyph_height;
    let out_w = cols
        .checked_mul(gw)
        .ok_or_else(|| CoreError::limit("output width overflow"))?;
    let out_h = rows
        .checked_mul(gh)
        .ok_or_else(|| CoreError::limit("output height overflow"))?;
    let bytes = crate::limits::checked_rgba_len(out_w, out_h, 4)?;
    let mut out = vec![0u8; bytes];

    for row in 0..rows {
        for col in 0..cols {
            let idx = (row * cols + col) as usize;
            let glyph = indices[idx];
            let fo = idx * 4;
            let (fr, fg_, fb, fa) = (
                fg[fo] as f32,
                fg[fo + 1] as f32,
                fg[fo + 2] as f32,
                fg[fo + 3] as f32 / 255.0,
            );
            let (br, bg_, bb, ba) = (
                bg[fo] as f32,
                bg[fo + 1] as f32,
                bg[fo + 2] as f32,
                bg[fo + 3] as f32 / 255.0,
            );
            for gy in 0..gh {
                for gx in 0..gw {
                    let ink = atlas.ink(glyph, gx, gy) as f32 / 255.0;
                    let la = fa * ink;
                    let a = la + ba * (1.0 - la);
                    let (r, g, b) = if a <= 0.0 {
                        (0.0, 0.0, 0.0)
                    } else {
                        (
                            (fr * la + br * ba * (1.0 - la)) / a,
                            (fg_ * la + bg_ * ba * (1.0 - la)) / a,
                            (fb * la + bb * ba * (1.0 - la)) / a,
                        )
                    };
                    let p = (((row * gh + gy) * out_w + col * gw + gx) * 4) as usize;
                    out[p] = clamp_round(r);
                    out[p + 1] = clamp_round(g);
                    out[p + 2] = clamp_round(b);
                    out[p + 3] = clamp_round(a * 255.0);
                }
            }
        }
    }
    Ok(out)
}

/// Convenience wrapper over `raster_cells` for a full conversion result.
/// (Used by native tests; the WASM API path goes through `raster_cells`.)
#[allow(dead_code)]
pub fn rasterize_ascii(
    data: &ConversionData,
    font: &VariantFont,
) -> Result<Vec<u8>> {
    raster_cells(
        data.grid.indices(),
        &data.cell_fg,
        &data.cell_bg,
        data.grid.cols(),
        data.grid.rows(),
        &font.atlas,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::{FontVariant, VariantFont};

    fn settings() -> ConvertSettings {
        ConvertSettings {
            dither_algorithm: DitherAlgorithm::None,
            dither_strength: 1.0,
            threshold: 0.5,
            cell_size: 8.0,
            color_mode: ColorMode::Mono,
            alpha_mode: AlphaMode::Ignore,
            contrast: 1.0,
            gamma: 1.0,
            target_width: 0,
            background: 0x000000FF,
            palette: vec![0x000000FF, 0xFFFFFFFF],
        }
    }

    fn solid(w: u32, h: u32, rgba: [u8; 4]) -> DecodedImage {
        let mut data = Vec::with_capacity((w * h * 4) as usize);
        for _ in 0..(w * h) {
            data.extend_from_slice(&rgba);
        }
        DecodedImage {
            rgba: data,
            width: w,
            height: h,
        }
    }

    #[test]
    fn grid_geometry_matches_contract() {
        let font = VariantFont::load(FontVariant::Unscii8).unwrap();
        let img = solid(32, 16, [128, 128, 128, 255]);
        let d = convert_frame(&img, &font, &settings()).unwrap();
        assert_eq!((d.grid.cols(), d.grid.rows()), (4, 2));
        assert_eq!(d.glyph_width, 8);
        assert_eq!(d.glyph_height, 8);
        assert_eq!(d.cell_fg.len(), 4 * 2 * 4);
        assert_eq!(d.cell_bg.len(), 4 * 2 * 4);
        let out_w = d.grid.cols() * d.glyph_width;
        let out_h = d.grid.rows() * d.glyph_height;
        assert_eq!((out_w, out_h), (32, 16));
        // 128/255 * 15 rounds to 8, same as the salvaged mapper.
        assert!(d.grid.indices().iter().all(|&v| v == 8));
    }

    #[test]
    fn tall_variant_preserves_source_aspect() {
        let font = VariantFont::load(FontVariant::Unscii8Tall).unwrap();
        assert_eq!((font.atlas.glyph_width, font.atlas.glyph_height), (8, 16));
        let img = solid(32, 16, [200, 200, 200, 255]);
        let s = settings(); // cell_size 8 -> box = 4x16 (aspect 8/16)
        let d = convert_frame(&img, &font, &s).unwrap();
        // cols = ceil(32/4)=8, rows = ceil(16/16)=1 -> output 64x16 (aspect 4:1 = source 32:16? no: 64/16=4 vs src 2) -- wait.
        // Source aspect 32:16 = 2. Output 8*8 : 1*16 = 64:16 = 4. Hmm!
        // box_w = cell * gw/gh = 8 * 0.5 = 4; out_w = ceil(32/4)*8 = 64.
        // out_h = ceil(16/16)*16 = 16. 64/16 = 4 != 2.
        // Rethink: for aspect preservation we need out_w/out_h == w/h.
        // out ≈ (W/box_w)*gw x (H/box_h)*gh = W*gw/(cell*gw/gh) x H*gh/cell
        //      = W*gh/cell x H*gh/cell -> ratio = W/H.
        // out_w = 32*16/8 = 64, out_h = 16*16/8 = 32? rows = ceil(16/8)...
        // box_h must be cell_size, and box_w = cell*gw/gh:
        //   rows = ceil(16/8) = 2 -> out_h = 2*16 = 32; cols = ceil(32/4)=8 -> out_w=64.
        //   64/32 = 2 == 32/16 ✓ (my settings() uses box_h = cell_size = 8, not 16!)
        assert_eq!((d.grid.cols(), d.grid.rows()), (8, 2));
        let aspect = (d.grid.cols() * d.glyph_width) as f64 / (d.grid.rows() * d.glyph_height) as f64;
        assert!((aspect - 2.0).abs() < 1e-9, "aspect {aspect}");
    }

    #[test]
    fn fractional_cell_size_partial_cells() {
        let font = VariantFont::load(FontVariant::Unscii8).unwrap();
        // 5x5 source, cell_size 2.5 -> box 2.5x2.5, cols=rows=2.
        let mut data = vec![0u8; 5 * 5 * 4];
        for (i, px) in data.chunks_exact_mut(4).enumerate() {
            let v = (i % 25) as u8 * 10;
            px[0] = v;
            px[1] = v;
            px[2] = v;
            px[3] = 255;
        }
        let img = DecodedImage {
            rgba: data,
            width: 5,
            height: 5,
        };
        let mut s = settings();
        s.cell_size = 2.5;
        let d = convert_frame(&img, &font, &s).unwrap();
        assert_eq!((d.grid.cols(), d.grid.rows()), (2, 2));
        // Cell (0,0) averages pixels x0..2 (4 px), cell (1,0) averages x2..5 (6 px).
        // Luma differs -> different glyph levels expected for this ramp input.
        let l00 = d.grid.get(0, 0).unwrap();
        let l10 = d.grid.get(1, 0).unwrap();
        assert_ne!(l00, l10);
        // Never out of bounds: raster must build cleanly.
        let _ = rasterize_ascii(&d, &font).unwrap();
    }

    #[test]
    fn target_width_resizes_exactly() {
        let font = VariantFont::load(FontVariant::Unscii8).unwrap();
        let img = solid(10, 4, [10, 20, 30, 255]);
        let mut s = settings();
        s.target_width = 6;
        s.cell_size = 1.0; // box 1x1
        let d = convert_frame(&img, &font, &s).unwrap();
        // Height = round(4 * 6/10) = 2 (>= 1).
        assert_eq!((d.sampled_width, d.sampled_height), (6, 2));
        assert_eq!((d.grid.cols(), d.grid.rows()), (6, 2));
        let mut s0 = settings();
        s0.target_width = 0;
        s0.cell_size = 1.0;
        let d0 = convert_frame(&img, &font, &s0).unwrap();
        assert_eq!((d0.sampled_width, d0.sampled_height), (10, 4));
    }

    #[test]
    fn alpha_modes_differ_end_to_end() {
        let font = VariantFont::load(FontVariant::Unscii8).unwrap();
        // Fully transparent pixel whose hidden RGB is white.
        let img = solid(8, 8, [255, 255, 255, 0]);

        // Ignore: white RGB used -> dense glyph.
        let mut s = settings();
        s.alpha_mode = AlphaMode::Ignore;
        let d = convert_frame(&img, &font, &s).unwrap();
        let ignore_glyph = d.grid.indices()[0];
        assert_eq!(ignore_glyph, 15);

        // Preserve: background (black) used for luma -> space glyph; cell bg alpha 0.
        let mut s = settings();
        s.alpha_mode = AlphaMode::Preserve;
        let d = convert_frame(&img, &font, &s).unwrap();
        assert_eq!(d.grid.indices()[0], 0);
        assert_eq!(d.cell_bg[3], 0);
        assert_eq!(d.cell_fg[3], 0); // maxA = 0 -> invisible ink

        // Blend over red background: luma of pure red composited result.
        let mut s = settings();
        s.alpha_mode = AlphaMode::Blend;
        s.background = 0xFF0000FF;
        let d = convert_frame(&img, &font, &s).unwrap();
        // Red luma = 0.2126*255 ~ 54 -> round(54/255*15) = round(3.19) = 3.
        assert_eq!(d.grid.indices()[0], 3);
        assert_eq!(&d.cell_bg[0..4], &[255, 0, 0, 255]);
    }

    #[test]
    fn color_and_palette_modes_sample_cell_colors() {
        let font = VariantFont::load(FontVariant::Unscii8).unwrap();
        let img = solid(8, 8, [200, 50, 25, 255]);
        let mut s = settings();
        s.color_mode = ColorMode::Color;
        let d = convert_frame(&img, &font, &s).unwrap();
        assert_eq!(&d.cell_fg[0..4], &[200, 50, 25, 255]);
        assert_eq!(&d.cell_bg[0..4], &[0, 0, 0, 255]);

        let mut s = settings();
        s.color_mode = ColorMode::Palette;
        s.palette = vec![0xFF0000FF, 0x00FF00FF, 0x0000FFFF, 0xFFFFFFFF];
        let d = convert_frame(&img, &font, &s).unwrap();
        // Nearest of pure-ish red/green/blue/white to (200,50,25) is red.
        assert_eq!(&d.cell_fg[0..4], &[255, 0, 0, 255]);
        // bg black nearest to first palette? black not in palette -> nearest among RGBA set: red? distances: black->red: redmean of (255,0,0) vs (0,0,0): dr=255: (2+127/256)*255^2 ≈ 65536... blue similar? (2+(255)/256... wait rm=(0+0)/2? For palette red entry vs black query: pr=255,r=0: rm=(255+0)/2=127: (2+127/256)*65025 + 0 + (2+(255-127)/256)*0 = (2.496)*65025 ≈ 162k; blue: pr=0,pb=255,b=0: rm=(0+0)/2=0 → term1=0, term3=(2+255/256)*65025 ≈ 131k; green like red → 162k? blue term3 = (2+0.996)*65025=194k?? hold on: green query component: g=0 pg=255: 4*65025=260k dominates for green. blue: db=255: (2+(255-0)/256)*65025 = (2.996)*65025 ≈ 194k. red: (2+127/256)*65025 = 2.496*65025 ≈ 162k. white: dr=dg=db=-255... all three: (2+rm/256 for r: rm=(255+0)/2=127 →2.496; g: rm=127→4*?? no white pg=255 g=0: rm=(255+0)/2 → 4*dg^2=260k; So red entry wins ≈162k. So bg → red (255,0,0). Eh, that's what nearest gives without black in palette.
        assert_eq!(&d.cell_bg[0..4], &[255, 0, 0, 255]);
    }

    #[test]
    fn contrast_gamma_neutral_identity() {
        let font = VariantFont::load(FontVariant::Unscii8).unwrap();
        let img = solid(16, 16, [37, 118, 201, 255]);
        let mut a = settings();
        let mut b = settings();
        b.contrast = 1.0;
        b.gamma = 1.0;
        let da = convert_frame(&img, &font, &a).unwrap();
        let db = convert_frame(&img, &font, &b).unwrap();
        assert_eq!(da.grid.indices(), db.grid.indices());
        // Non-neutral contrast changes the level for a mid-low color.
        a.contrast = 2.0;
        let dc = convert_frame(&img, &font, &a).unwrap();
        // Not asserting inequality of glyph (may coincide); assert sampled luma changes via raster path instead: compare cell fg in Color mode.
        a.color_mode = ColorMode::Color;
        let de = convert_frame(&img, &font, &a).unwrap();
        b.color_mode = ColorMode::Color;
        let df = convert_frame(&img, &font, &b).unwrap();
        assert_ne!(&de.cell_fg[0..3], &df.cell_fg[0..3]);
        let _ = dc;
    }

    #[test]
    fn invalid_params_rejected() {
        let font = VariantFont::load(FontVariant::Unscii8).unwrap();
        let img = solid(8, 8, [0, 0, 0, 255]);
        let mut s = settings();
        s.cell_size = 0.0;
        assert!(convert_frame(&img, &font, &s).is_err());
        let mut s = settings();
        s.cell_size = f32::NAN;
        assert!(convert_frame(&img, &font, &s).is_err());
        let mut s = settings();
        s.contrast = 0.0; // below 0.1
        assert!(convert_frame(&img, &font, &s).is_err());
        let mut s = settings();
        s.color_mode = ColorMode::Palette;
        s.palette = vec![];
        assert!(convert_frame(&img, &font, &s).is_err());
    }
}
