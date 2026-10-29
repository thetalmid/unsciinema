//! Font atlas construction: 16 ramp glyphs laid out horizontally.
//!
//! Atlas RGBA convention (matches the renderer and the CPU reference raster):
//! - ink pixel:   R=G=B=A=255
//! - empty pixel: R=G=B=0, A=255
//! Coverage for compositing is read from the R channel (0 or 1 today; the
//! layout would support antialiased coverage unchanged).

use crate::error::{CoreError, Result};
use crate::font::parser::Glyph;

#[derive(Debug, Clone)]
pub struct FontAtlas {
    pub width: u32,
    pub height: u32,
    pub glyph_width: u32,
    pub glyph_height: u32,
    pub glyph_count: u32,
    /// RGBA8, `width * height * 4` bytes.
    pub data: Vec<u8>,
}

impl FontAtlas {
    /// Build an atlas from exactly 16 uniformly-sized glyphs.
    pub fn from_glyphs(glyphs: &[Glyph]) -> Result<FontAtlas> {
        const GLYPH_COUNT: u32 = 16;
        if glyphs.len() != GLYPH_COUNT as usize {
            return Err(CoreError::font(format!(
                "expected 16 glyphs, got {}",
                glyphs.len()
            )));
        }
        let glyph_w = glyphs[0].width;
        let glyph_h = glyphs[0].height;
        if glyph_w == 0 || glyph_h == 0 || glyph_w > 64 || glyph_h > 64 {
            return Err(CoreError::font(format!(
                "unreasonable glyph dimensions {glyph_w}x{glyph_h}"
            )));
        }
        for g in glyphs {
            if g.width != glyph_w || g.height != glyph_h {
                return Err(CoreError::font(format!(
                    "mixed glyph dimensions: {glyph_w}x{glyph_h} vs {}x{}",
                    g.width, g.height
                )));
            }
        }

        let atlas_width = GLYPH_COUNT * glyph_w;
        let atlas_height = glyph_h;
        let mut data = vec![0u8; (atlas_width as usize) * (atlas_height as usize) * 4];

        for (idx, glyph) in glyphs.iter().enumerate() {
            let base_x = idx as u32 * glyph_w;
            for gy in 0..glyph_h {
                for gx in 0..glyph_w {
                    let val = if glyph.get_pixel(gx, gy) { 255 } else { 0 };
                    let p = ((gy * atlas_width + base_x + gx) * 4) as usize;
                    data[p] = val;
                    data[p + 1] = val;
                    data[p + 2] = val;
                    data[p + 3] = 255;
                }
            }
        }

        Ok(FontAtlas {
            width: atlas_width,
            height: atlas_height,
            glyph_width: glyph_w,
            glyph_height: glyph_h,
            glyph_count: GLYPH_COUNT,
            data,
        })
    }

    /// Ink coverage (0 or 1) of glyph `idx` at local pixel (x, y).
    pub fn ink(&self, idx: u16, x: u32, y: u32) -> u8 {
        if idx as u32 >= self.glyph_count || x >= self.glyph_width || y >= self.glyph_height {
            return 0;
        }
        let ax = idx as u32 * self.glyph_width + x;
        let p = ((y * self.width + ax) * 4) as usize;
        self.data.get(p).copied().unwrap_or(0)
    }
}
