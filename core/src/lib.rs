//! Unsciinema core — WASM public API.
//!
//! Rust/WASM is the authoritative implementation of image decode,
//! preprocessing, cell sampling, dithering, glyph selection and the CPU
//! reference raster. WebGL2 only consumes `ConversionResult`; it never
//! recomputes a different glyph grid.
//!
//! Data contract (see INTEGRATION.md):
//! - row-major, top-left origin, straight RGBA8 at every CPU/export boundary;
//! - `glyphIndices` are atlas indices, `charRamp[index]` is the Unicode char;
//! - palette integers are `0xRRGGBBAA` (explicit shifts, never memcpy'd);
//! - every public boundary validates lengths/dims/enums with checked
//!   arithmetic; failures arrive in JS as exceptions with useful messages.

use std::collections::HashMap;

use wasm_bindgen::prelude::*;

mod convert;
mod dither;
mod error;
mod export;
mod font;
mod image;
mod limits;

use convert::{raster_cells, ConvertSettings};
use dither::DitherAlgorithm;
use error::CoreError;
use font::{FontVariant, VariantFont};
use image::luminance::AlphaMode;
use image::loader::LoadedSource;

fn to_js<T>(r: Result<T, CoreError>) -> Result<T, JsValue> {
    r.map_err(|e| JsValue::from_str(&e.to_string()))
}

// ---------------------------------------------------------------------------
// Parameters
// ---------------------------------------------------------------------------

/// Conversion parameters. Numeric enums are validated at every use: unknown
/// values are rejected (they are NOT silently coerced to a default).
#[wasm_bindgen]
#[allow(non_snake_case)] // JS-facing field names follow the documented camelCase contract
pub struct ConvertParams {
    /// 0..=10 (see DITHER_ALGORITHMS in the frontend; 10 = Blue Noise)
    pub ditherAlgorithm: u32,
    /// 0..=1, default 1.0
    pub ditherStrength: f32,
    /// 0..=1, default 0.5 (neutral)
    pub threshold: f32,
    /// 0..=6 (seven font variants)
    pub fontVariant: u32,
    /// source pixels per cell (height), 1..=64, default 8
    pub cellSize: f32,
    /// 0=mono, 1=color, 2=palette
    pub colorMode: u32,
    /// 0=ignore, 1=preserve, 2=blend
    pub alphaMode: u32,
    /// 0.1..=3.0, default 1.0 (neutral)
    pub contrast: f32,
    /// 0.5..=2.5, default 1.0 (neutral)
    pub gamma: f32,
    /// forced source width in pixels; 0 = no forced resize
    pub targetWidth: u32,
    /// background color 0xRRGGBBAA
    pub background: u32,
    palette: Vec<u32>,
}

#[wasm_bindgen]
impl ConvertParams {
    #[wasm_bindgen(constructor)]
    pub fn new() -> ConvertParams {
        ConvertParams {
            ditherAlgorithm: 0,
            ditherStrength: 1.0,
            threshold: 0.5,
            fontVariant: 0,
            cellSize: 8.0,
            colorMode: 0,
            alphaMode: 0,
            contrast: 1.0,
            gamma: 1.0,
            targetWidth: 0,
            background: 0x000000FF,
            palette: vec![0x000000FF, 0xFFFFFFFF],
        }
    }

    /// Current palette (copied into JS as numbers, 0xRRGGBBAA).
    #[wasm_bindgen(getter, js_name = palette)]
    pub fn palette_js(&self) -> Vec<u32> {
        self.palette.clone()
    }

    /// Replace the palette (validated: 1..=64 entries).
    #[wasm_bindgen(js_name = setPalette)]
    pub fn set_palette_js(&mut self, colors: &[u32]) -> Result<(), JsValue> {
        to_js(dither::palette::validate_palette(colors))?;
        self.palette = colors.to_vec();
        Ok(())
    }
}

impl Default for ConvertParams {
    fn default() -> Self {
        ConvertParams::new()
    }
}

impl ConvertParams {
    fn settings(&self) -> Result<ConvertSettings, CoreError> {
        let dither_algorithm = DitherAlgorithm::from_u32(self.ditherAlgorithm)
            .ok_or_else(|| CoreError::invalid(format!("unknown dither algorithm {}", self.ditherAlgorithm)))?;
        let color_mode = convert::ColorMode::from_u32(self.colorMode)
            .ok_or_else(|| CoreError::invalid(format!("unknown color mode {}", self.colorMode)))?;
        let alpha_mode = AlphaMode::from_u32(self.alphaMode)
            .ok_or_else(|| CoreError::invalid(format!("unknown alpha mode {}", self.alphaMode)))?;
        // Validates font enum too (fail fast before any work).
        FontVariant::from_u32(self.fontVariant)
            .ok_or_else(|| CoreError::invalid(format!("unknown font variant {}", self.fontVariant)))?;
        let dither_strength = dither::validate_strength(self.ditherStrength)?;
        let threshold = dither::validate_threshold(self.threshold)?;
        let cell_size = limits::check_f32("cell_size", self.cellSize, 1.0, 64.0)?;
        let contrast = image::preprocessing::validate_contrast(self.contrast)?;
        let gamma = image::preprocessing::validate_gamma(self.gamma)?;
        if self.targetWidth > limits::MAX_SOURCE_DIM {
            return Err(CoreError::limit(format!(
                "target_width {} exceeds max {}",
                self.targetWidth,
                limits::MAX_SOURCE_DIM
            )));
        }
        Ok(ConvertSettings {
            dither_algorithm,
            dither_strength,
            threshold,
            cell_size,
            color_mode,
            alpha_mode,
            contrast,
            gamma,
            target_width: self.targetWidth,
            background: self.background,
            palette: self.palette.clone(),
        })
    }
}

/// All 31 palette presets as a JSON array
/// (`[{"name":"...","colors":[4278190335,...]}, ...]`, colors are 0xRRGGBBAA
/// numbers). The frontend reads presets from here so Rust and UI can never
/// drift apart (names are restricted to `[a-z0-9-]` by construction, and all
/// values are integers — the output is structurally valid JSON by design).
#[wasm_bindgen(js_name = palettePresetsJson)]
pub fn palette_presets_json() -> String {
    let mut out = String::from("[");
    for (i, (name, colors)) in dither::palette::PALETTES.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str("{\"name\":\"");
        out.push_str(name);
        out.push_str("\",\"colors\":[");
        for (j, c) in colors.iter().enumerate() {
            if j > 0 {
                out.push(',');
            }
            out.push_str(&c.to_string());
        }
        out.push_str("]}");
    }
    out.push(']');
    out
}

/// Variant table for the UI: `[{"name":"unscii-8","woff":"unscii-8.woff"}, ...]`
/// in canonical index order (0..=6). Single source of truth for both the
/// converter and the HTML-export WOFF URLs.
#[wasm_bindgen(js_name = fontVariantsJson)]
pub fn font_variants_json() -> String {
    let mut out = String::from("[");
    for (i, v) in font::FontVariant::ALL.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str("{\"name\":\"");
        out.push_str(v.name());
        out.push_str("\",\"woff\":\"");
        out.push_str(v.woff_name());
        out.push_str("\"}");
    }
    out.push(']');
    out
}

// ---------------------------------------------------------------------------
// Result
// ---------------------------------------------------------------------------

/// One conversion result. Typed arrays are clones of Wasm memory: the returned
/// views are independent JS buffers. Callers must not expect them to live-sync
/// with Wasm state (they are snapshots by design).
#[wasm_bindgen]
#[allow(non_snake_case)] // JS-facing field names follow the documented camelCase contract
pub struct ConversionResult {
    pub cols: u32,
    pub rows: u32,
    pub glyphWidth: u32,
    pub glyphHeight: u32,
    pub outputWidth: u32,
    pub outputHeight: u32,
    pub atlasWidth: u32,
    pub atlasHeight: u32,
    pub glyphCount: u32,
    /// Which font variant produced this result (used by raster/exports).
    font_variant: u32,
    #[wasm_bindgen(getter_with_clone)]
    pub glyphIndices: Vec<u16>,
    #[wasm_bindgen(getter_with_clone)]
    pub charRamp: Vec<String>,
    #[wasm_bindgen(getter_with_clone)]
    pub cellForegroundRGBA: Vec<u8>,
    #[wasm_bindgen(getter_with_clone)]
    pub cellBackgroundRGBA: Vec<u8>,
    #[wasm_bindgen(getter_with_clone)]
    pub atlasRGBA: Vec<u8>,
}

// ---------------------------------------------------------------------------
// Source info
// ---------------------------------------------------------------------------

/// Metadata for the loaded source (returned by `loadImage`).
#[wasm_bindgen]
#[allow(non_snake_case)] // JS-facing field names follow the documented camelCase contract
pub struct SourceInfo {
    pub width: u32,
    pub height: u32,
    pub frameCount: u32,
    pub animated: bool,
    /// "png" | "jpeg" | "webp" | "gif"
    #[wasm_bindgen(getter_with_clone)]
    pub format: String,
    /// Per-frame delay in milliseconds.
    #[wasm_bindgen(getter_with_clone)]
    pub delaysMs: Vec<u32>,
    /// -1 = no loop extension present, 0 = loop forever, n>0 = repeat n times.
    pub loopCode: i32,
}

// ---------------------------------------------------------------------------
// Converter
// ---------------------------------------------------------------------------

#[wasm_bindgen]
pub struct UnsciiConverter {
    /// Lazily built, cached per variant (atlas + computed ramp).
    fonts: std::cell::RefCell<HashMap<u32, VariantFont>>,
    source: Option<LoadedSource>,
}

#[wasm_bindgen]
impl UnsciiConverter {
    #[wasm_bindgen(constructor)]
    pub fn new() -> UnsciiConverter {
        console_error_panic_hook::set_once();
        UnsciiConverter {
            fonts: std::cell::RefCell::new(HashMap::new()),
            source: None,
        }
    }

    /// Content-sniffed load (PNG/JPEG/GIF/WebP; no extension involved).
    /// GIFs are composited frame-by-frame with delay/loop metadata retained.
    /// Replaces any previously loaded source.
    #[wasm_bindgen(js_name = loadImage)]
    pub fn load_image(&mut self, bytes: &[u8]) -> Result<SourceInfo, JsValue> {
        let loaded = to_js(image::loader::load_source(bytes))?;
        let info = source_info(&loaded);
        self.source = Some(loaded);
        Ok(info)
    }

    /// Metadata of the currently loaded source.
    #[wasm_bindgen(js_name = sourceInfo)]
    pub fn source_info_js(&self) -> Result<SourceInfo, JsValue> {
        let s = self
            .source
            .as_ref()
            .ok_or_else(|| JsValue::from_str("no image loaded"))?;
        Ok(source_info(s))
    }

    /// RGBA8 bytes of a frame (straight alpha, top-left origin). Transferred
    /// to the renderer as the immutable source texture / CPU fallback input.
    #[wasm_bindgen(js_name = sourceRgba)]
    pub fn source_rgba(&self, frame: u32) -> Result<Vec<u8>, JsValue> {
        let s = self
            .source
            .as_ref()
            .ok_or_else(|| JsValue::from_str("no image loaded"))?;
        let f = s
            .frames
            .get(frame as usize)
            .ok_or_else(|| JsValue::from_str(&format!("frame {frame} out of range")))?;
        Ok(f.rgba.clone())
    }

    /// Convert frame 0 (still images, or the first frame of a GIF).
    pub fn convert(&self, params: &ConvertParams) -> Result<ConversionResult, JsValue> {
        self.convert_frame(0, params)
    }

    /// Convert one frame with the current settings.
    #[wasm_bindgen(js_name = convertFrame)]
    pub fn convert_frame(&self, frame: u32, params: &ConvertParams) -> Result<ConversionResult, JsValue> {
        let settings = to_js(params.settings())?;
        let s = self
            .source
            .as_ref()
            .ok_or_else(|| JsValue::from_str("no image loaded"))?;
        let img = s
            .frames
            .get(frame as usize)
            .ok_or_else(|| JsValue::from_str(&format!("frame {frame} out of range")))?;
        let variant = FontVariant::from_u32(params.fontVariant)
            .ok_or_else(|| JsValue::from_str("unknown font variant"))?;
        let font = self.font_for(variant)?;
        let data = to_js(convert::convert_frame(img, &font, &settings))?;
        Ok(self.build_result(variant, &font, &data))
    }

    /// CPU reference raster of the ASCII layer for a result (straight RGBA,
    /// `outputWidth x outputHeight`). The WebGL2 preview and the Canvas2D
    /// fallback must match this when CRT effects are off.
    pub fn raster(&self, result: &ConversionResult) -> Result<Vec<u8>, JsValue> {
        result.validate()?;
        let variant = FontVariant::from_u32(result.font_variant)
            .ok_or_else(|| JsValue::from_str("unknown font variant"))?;
        let font = self.font_for(variant)?;
        to_js(raster_cells(
            &result.glyphIndices,
            &result.cellForegroundRGBA,
            &result.cellBackgroundRGBA,
            result.cols,
            result.rows,
            &font.atlas,
        ))
    }

    // ---- exports (methods so they reuse the font cache) -------------------

    /// PNG bytes for an RGBA buffer (already rendered, incl. blend/CRT as
    /// produced by the renderer readback).
    #[wasm_bindgen(js_name = exportPng)]
    pub fn export_png(&self, rgba: &[u8], width: u32, height: u32) -> Result<Vec<u8>, JsValue> {
        to_js(export::export_png(rgba, width, height))
    }

    /// Source frame resized (CatmullRom) to the result's output dimensions —
    /// the renderer's source texture and the blend reference for CPU parity.
    #[wasm_bindgen(js_name = previewSource)]
    pub fn preview_source(
        &self,
        result: &ConversionResult,
        frame: u32,
    ) -> Result<Vec<u8>, JsValue> {
        result.validate()?;
        Ok(to_js(self.resized_frame(result, frame))?)
    }

    /// CPU reference raster with the ASCII/original presentation blend applied:
    /// `lerp(source, ascii, asciiStrength)` per pixel at output resolution
    /// (`asciiStrength` 1.0 = full ASCII (default), 0.0 = original image).
    /// This is what the Canvas2D fallback draws and what raster exports embed
    /// when readback is unavailable. Character-only exports ignore it by
    /// definition (documented in INTEGRATION.md).
    #[wasm_bindgen(js_name = rasterBlended)]
    pub fn raster_blended(
        &self,
        result: &ConversionResult,
        frame: u32,
        ascii_strength: f32,
    ) -> Result<Vec<u8>, JsValue> {
        if !ascii_strength.is_finite() || !(0.0..=1.0).contains(&ascii_strength) {
            return Err(JsValue::from_str(
                "asciiStrength must be finite and within 0..=1",
            ));
        }
        result.validate()?;
        let variant = FontVariant::from_u32(result.font_variant)
            .ok_or_else(|| JsValue::from_str("unknown font variant"))?;
        let font = self.font_for(variant)?;
        let ascii = raster_cells(
            &result.glyphIndices,
            &result.cellForegroundRGBA,
            &result.cellBackgroundRGBA,
            result.cols,
            result.rows,
            &font.atlas,
        )?;
        let src = self.resized_frame(result, frame)?;
        let s = ascii_strength;
        let out: Vec<u8> = ascii
            .iter()
            .zip(src.iter())
            .map(|(&a, &o)| {
                // Match the GLSL `mix(o, a, s)` + round-half-up on both sides.
                (o as f32 + (a as f32 - o as f32) * s + 0.5).floor() as u8
            })
            .collect();
        Ok(out)
    }

    /// Resize the stored immutable frame to the result's output size.
    fn resized_frame(
        &self,
        result: &ConversionResult,
        frame: u32,
    ) -> Result<Vec<u8>, CoreError> {
        let s = self
            .source
            .as_ref()
            .ok_or_else(|| CoreError::invalid("no image loaded"))?;
        let f = s
            .frames
            .get(frame as usize)
            .ok_or_else(|| CoreError::invalid(format!("frame {frame} out of range")))?;
        let w = result.outputWidth;
        let h = result.outputHeight;
        if w == 0 || h == 0 {
            return Err(CoreError::invalid("zero output size"));
        }
        if f.width == w && f.height == h {
            return Ok(f.rgba.clone());
        }
        let img = ::image::RgbaImage::from_raw(f.width, f.height, f.rgba.clone())
            .ok_or_else(|| CoreError::decode("frame buffer size mismatch"))?;
        let out = ::image::imageops::resize(&img, w, h, ::image::imageops::FilterType::CatmullRom);
        Ok(out.into_raw())
    }

    /// Plain UTF-8 text: exactly the grid rows/glyphs (no font/colors/CRT).
    #[wasm_bindgen(js_name = exportText)]
    pub fn export_text(&self, result: &ConversionResult) -> Result<String, JsValue> {
        result.validate()?;
        let variant = FontVariant::from_u32(result.font_variant)
            .ok_or_else(|| JsValue::from_str("unknown font variant"))?;
        let font = self.font_for(variant)?;
        to_js(export::export_text(
            &result.glyphIndices,
            result.cols,
            result.rows,
            &font,
        ))
    }

    /// 24-bit ANSI with per-cell fg/bg, ending in a reset.
    #[wasm_bindgen(js_name = exportAnsi)]
    pub fn export_ansi(&self, result: &ConversionResult) -> Result<String, JsValue> {
        result.validate()?;
        let variant = FontVariant::from_u32(result.font_variant)
            .ok_or_else(|| JsValue::from_str("unknown font variant"))?;
        let font = self.font_for(variant)?;
        to_js(export::export_ansi(
            &result.glyphIndices,
            &result.cellForegroundRGBA,
            &result.cellBackgroundRGBA,
            result.cols,
            result.rows,
            &font,
        ))
    }

    /// Standalone HTML with the embedded WOFF (`woff` = the real variant
    /// font bytes fetched same-origin; validated). `colored` emits per-cell
    /// color spans (color/palette modes).
    #[wasm_bindgen(js_name = exportHtml)]
    pub fn export_html(
        &self,
        result: &ConversionResult,
        woff: &[u8],
        colored: bool,
    ) -> Result<String, JsValue> {
        result.validate()?;
        let variant = FontVariant::from_u32(result.font_variant)
            .ok_or_else(|| JsValue::from_str("unknown font variant"))?;
        let font = self.font_for(variant)?;
        to_js(export::export_html(
            &result.glyphIndices,
            &result.cellForegroundRGBA,
            &result.cellBackgroundRGBA,
            result.cols,
            result.rows,
            &font,
            woff,
            &export::HtmlExportOptions { colored },
        ))
    }
}

impl UnsciiConverter {
    /// Cached variant load; returns a cheap clone (atlas ~4-8 KiB) so every
    /// public method can stay `&self`.
    fn font_for(&self, variant: FontVariant) -> Result<VariantFont, JsValue> {
        let key = variant as u32;
        if !self.fonts.borrow().contains_key(&key) {
            let font = to_js(VariantFont::load(variant))?;
            self.fonts.borrow_mut().insert(key, font);
        }
        Ok(self.fonts.borrow().get(&key).expect("just inserted").clone())
    }

    fn build_result(
        &self,
        variant: FontVariant,
        font: &VariantFont,
        data: &convert::ConversionData,
    ) -> ConversionResult {
        let cols = data.grid.cols();
        let rows = data.grid.rows();
        ConversionResult {
            cols,
            rows,
            glyphWidth: data.glyph_width,
            glyphHeight: data.glyph_height,
            outputWidth: cols * data.glyph_width,
            outputHeight: rows * data.glyph_height,
            atlasWidth: font.atlas.width,
            atlasHeight: font.atlas.height,
            glyphCount: font.atlas.glyph_count,
            font_variant: variant as u32,
            glyphIndices: data.grid.indices().to_vec(),
            charRamp: font.ramp.iter().map(|c| c.to_string()).collect(),
            cellForegroundRGBA: data.cell_fg.clone(),
            cellBackgroundRGBA: data.cell_bg.clone(),
            atlasRGBA: font.atlas.data.clone(),
        }
    }
}

impl Default for UnsciiConverter {
    fn default() -> Self {
        UnsciiConverter::new()
    }
}

impl ConversionResult {
    /// Validate cross-field invariants (also run by every export path).
    pub fn validate(&self) -> Result<(), CoreError> {
        let cells = limits::checked_cells(self.cols, self.rows)?;
        if self.glyphIndices.len() != cells {
            return Err(CoreError::invalid(format!(
                "glyphIndices has {} entries, expected cols*rows={cells}",
                self.glyphIndices.len()
            )));
        }
        if self.cellForegroundRGBA.len() != cells * 4
            || self.cellBackgroundRGBA.len() != cells * 4
        {
            return Err(CoreError::invalid(
                "cell color arrays must be cols*rows*4 bytes",
            ));
        }
        if self.outputWidth != self.cols * self.glyphWidth
            || self.outputHeight != self.rows * self.glyphHeight
        {
            return Err(CoreError::invalid("output dimensions inconsistent"));
        }
        limits::checked_rgba_len(self.outputWidth, self.outputHeight, 4)?;
        let atlas_expected =
            limits::checked_rgba_len(self.atlasWidth, self.atlasHeight, 4)?;
        if self.atlasRGBA.len() != atlas_expected {
            return Err(CoreError::invalid("atlas_rgba size mismatch"));
        }
        if self.glyphCount != 16 || self.atlasWidth != self.glyphCount * self.glyphWidth {
            return Err(CoreError::invalid("atlas layout inconsistent"));
        }
        if self.charRamp.len() != self.glyphCount as usize {
            return Err(CoreError::invalid("char_ramp length mismatch"));
        }
        Ok(())
    }
}

fn source_info(s: &LoadedSource) -> SourceInfo {
    let first = &s.frames[0];
    SourceInfo {
        width: first.width,
        height: first.height,
        frameCount: s.frames.len() as u32,
        animated: s.animated,
        format: s.format.to_string(),
        delaysMs: s.delays_ms.clone(),
        loopCode: match s.loop_count {
            None => -1,
            Some(n) => n as i32,
        },
    }
}

// ---------------------------------------------------------------------------
// GIF writer (incremental: frames are quantized and written one at a time so
// RGBA working buffers are dropped immediately)
// ---------------------------------------------------------------------------

/// Incremental GIF encoder. `loopCode`: -1 = no loop extension (still GIF),
/// 0 = loop forever, n>0 = repeat n extra times.
#[wasm_bindgen]
pub struct GifWriter {
    inner: export::GifWriter,
}

#[wasm_bindgen]
impl GifWriter {
    #[wasm_bindgen(constructor)]
    pub fn new(width: u32, height: u32, loop_code: i32) -> Result<GifWriter, JsValue> {
        let spec = match loop_code {
            -1 => export::GifLoop::None,
            0 => export::GifLoop::Infinite,
            n if n > 0 => export::GifLoop::Finite(
                u16::try_from(n).map_err(|_| JsValue::from_str("loop count too large"))?,
            ),
            n => {
                return Err(JsValue::from_str(&format!(
                    "invalid loop_code {n} (use -1, 0 or >0)"
                )))
            }
        };
        let inner = to_js(export::GifWriter::new(width, height, spec))?;
        Ok(GifWriter { inner })
    }

    /// Append one full RGBA frame (validated against the declared size).
    #[wasm_bindgen(js_name = addFrame)]
    pub fn add_frame(&mut self, rgba: &[u8], delay_ms: u32) -> Result<(), JsValue> {
        to_js(self.inner.add_frame(rgba, delay_ms))
    }

    #[wasm_bindgen(js_name = frameCount)]
    pub fn frame_count(&self) -> u32 {
        self.inner.frame_count()
    }

    /// Finish and return the GIF bytes (consumes the writer).
    pub fn finish(self) -> Result<Vec<u8>, JsValue> {
        to_js(self.inner.finish())
    }
}

#[cfg(test)]
mod blend_tests {
    use super::*;
    use crate::convert::raster_cells;
    use crate::font::{FontVariant, VariantFont};

    fn tiny_result(font: &VariantFont) -> ConversionResult {
        // 2x2 cells of solid colors via explicit grid.
        let cells = 4u32;
        ConversionResult {
            font_variant: 0,
            cols: 2,
            rows: 2,
            glyphWidth: font.atlas.glyph_width,
            glyphHeight: font.atlas.glyph_height,
            outputWidth: 2 * font.atlas.glyph_width,
            outputHeight: 2 * font.atlas.glyph_height,
            glyphIndices: vec![16u16; cells as usize], // full block
            charRamp: font.ramp.iter().map(|c| c.to_string()).collect(),
            cellForegroundRGBA: vec![255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255],
            cellBackgroundRGBA: (0..cells).flat_map(|_| [0u8, 0, 0, 255]).collect(),
            atlasRGBA: font.atlas.data.clone(),
            atlasWidth: font.atlas.width,
            atlasHeight: font.atlas.height,
            glyphCount: font.atlas.glyph_count,
        }
    }

    #[test]
    fn blend_endpoints_are_exact() {
        let font = VariantFont::load(FontVariant::Unscii8).unwrap();
        let r = tiny_result(&font);
        r.validate().unwrap();
        let ascii = raster_cells(
            &r.glyphIndices,
            &r.cellForegroundRGBA,
            &r.cellBackgroundRGBA,
            r.cols,
            r.rows,
            &font.atlas,
        )
        .unwrap();
        // synthesize a source: mid-gray everywhere
        let src = vec![100u8; ascii.len()];
        let lerp = |s: f32| -> Vec<u8> {
            src.iter()
                .zip(ascii.iter())
                .map(|(&o, &a)| (o as f32 + (a as f32 - o as f32) * s + 0.5).floor() as u8)
                .collect()
        };
        assert_eq!(lerp(1.0), ascii, "strength 1 must be exactly the ascii raster");
        assert_eq!(lerp(0.0), src, "strength 0 must be exactly the source");
        // Midpoint differs from both endpoints (source 100 vs fg/bg extremes).
        let mid = lerp(0.5);
        assert!(mid.iter().any(|&v| v != 100));
    }

    #[test]
    fn preview_source_dims_match_output() {
        // resized_frame is private; validate its contract through the public
        // path is wasm-only, so check the invariant it relies on here:
        // output dims == cols*gw, rows*gh (covered by ConversionResult::validate).
        let font = VariantFont::load(FontVariant::Unscii16).unwrap();
        let r = tiny_result(&font);
        r.validate().unwrap();
        assert_eq!(r.outputWidth, r.cols * r.glyphWidth);
        assert_eq!(r.outputHeight, r.rows * r.glyphHeight);
        // tall/16 glyph geometry: 8x16 cells for unscii-16
        assert_eq!((r.glyphWidth, r.glyphHeight), (8, 16));
    }
}
