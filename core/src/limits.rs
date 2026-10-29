//! Shared input limits and validation helpers.
//!
//! Every limit is checked *before* any large allocation. WASM linear memory
//! cannot be recovered from an OOM abort, so the boundary must reject
//! oversized inputs up front with a useful message.

/// Maximum encoded input size accepted for any file (48 MiB).
pub const MAX_ENCODED_BYTES: usize = 48 * 1024 * 1024;
/// Maximum width/height of a decoded still source image.
pub const MAX_SOURCE_DIM: u32 = 8192;
/// Maximum pixel count of a decoded still source image (12 MP).
pub const MAX_SOURCE_PIXELS: u64 = 12_000_000;
/// Maximum allocation the `image` crate may request for one decoded image (64 MiB).
pub const MAX_IMAGE_ALLOC: u64 = 64 * 1024 * 1024;
/// Maximum number of glyph cells in one conversion grid.
pub const MAX_CELLS: u64 = 4_000_000;
/// Maximum output raster dimension (matches common WebGL texture limits).
pub const MAX_OUTPUT_DIM: u32 = 8192;
/// Maximum output raster pixels (16.7 MP).
pub const MAX_OUTPUT_PIXELS: u64 = 16_777_216;
/// Maximum frames accepted from an animated GIF.
pub const MAX_GIF_FRAMES: u32 = 64;
/// Maximum cumulative pixels across all GIF frames (12 MP -> ~48 MB RGBA).
pub const MAX_GIF_TOTAL_PIXELS: u64 = 12_000_000;
/// Maximum cumulative GIF frame delay (120 seconds).
pub const MAX_GIF_TOTAL_DELAY_MS: u64 = 120_000;
/// Maximum cells for colored HTML export (keeps the document bounded).
pub const MAX_HTML_COLORED_CELLS: u64 = 250_000;

use crate::error::CoreError;

/// Validate that two dimensions can back a byte buffer of `channels` bytes.
pub fn checked_rgba_len(width: u32, height: u32, channels: usize) -> Result<usize, CoreError> {
    if width == 0 || height == 0 {
        return Err(CoreError::invalid(format!(
            "zero-sized image ({width}x{height})"
        )));
    }
    if width > MAX_OUTPUT_DIM || height > MAX_OUTPUT_DIM {
        return Err(CoreError::limit(format!(
            "dimension {width}x{height} exceeds max {MAX_OUTPUT_DIM}"
        )));
    }
    let pixels = (width as u64)
        .checked_mul(height as u64)
        .ok_or_else(|| CoreError::limit("dimension overflow"))?;
    if pixels > MAX_OUTPUT_PIXELS {
        return Err(CoreError::limit(format!(
            "{pixels} pixels exceeds max {MAX_OUTPUT_PIXELS}"
        )));
    }
    let bytes = pixels
        .checked_mul(channels as u64)
        .ok_or_else(|| CoreError::limit("byte size overflow"))?;
    Ok(bytes as usize)
}

/// Validate a grid (cols, rows) and its flat length.
pub fn checked_cells(cols: u32, rows: u32) -> Result<usize, CoreError> {
    if cols == 0 || rows == 0 {
        return Err(CoreError::invalid(format!("empty grid {cols}x{rows}")));
    }
    let cells = (cols as u64)
        .checked_mul(rows as u64)
        .ok_or_else(|| CoreError::limit("grid overflow"))?;
    if cells > MAX_CELLS {
        return Err(CoreError::limit(format!(
            "{cells} cells exceeds max {MAX_CELLS}"
        )));
    }
    Ok(cells as usize)
}

/// Validate a f32 parameter: finite and within an inclusive range.
pub fn check_f32(name: &str, v: f32, min: f32, max: f32) -> Result<f32, CoreError> {
    if !v.is_finite() {
        return Err(CoreError::invalid(format!("{name} is not finite")));
    }
    if v < min || v > max {
        return Err(CoreError::invalid(format!(
            "{name}={v} outside [{min}, {max}]"
        )));
    }
    Ok(v)
}
