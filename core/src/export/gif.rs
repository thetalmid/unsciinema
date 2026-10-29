//! Animated/still GIF export via the `gif` crate (MIT OR Apache-2.0).
//!
//! Quantization policy (documented in INTEGRATION.md):
//! - pixels with alpha < 128 map to a reserved **transparent index**;
//! - opaque pixels (alpha >= 128) are quantized with NeuQuant (`color_quant`,
//!   MIT) — RGB is taken straight (no premultiplication), alpha is binary in
//!   GIF, so partial alpha becomes opaque unless below the 128 cut;
//! - when transparency exists NeuQuant builds a 255-color palette and index
//!   255 is reserved for transparency; otherwise 256 colors are used;
//! - every frame is a full independent composited frame and is written with
//!   `DisposalMethod::Background` so transparent pixels never leak the
//!   previous frame's content.
//!
//! Timing/delay: `delay_ms` is converted to GIF centiseconds with rounding
//! (`(ms + 5) / 10`), preserving 0 as 0. Loop metadata is whatever the caller
//! passes: `None` = omit the NETSCAPE extension (still images), `Some(0)` =
//! loop forever, `Some(n)` = repeat n extra times.

use crate::error::{CoreError, Result};
use crate::limits::checked_rgba_len;
use color_quant::NeuQuant;

pub enum GifLoop {
    /// No loop extension (plays once in spec-compliant players).
    None,
    Infinite,
    Finite(u16),
}

pub struct GifWriter {
    width: u16,
    height: u16,
    encoder: gif::Encoder<Vec<u8>>,
    frames: u32,
}

impl GifWriter {
    pub fn new(width: u32, height: u32, loop_spec: GifLoop) -> Result<GifWriter> {
        let expected = checked_rgba_len(width, height, 4)?;
        if expected == 0 {
            return Err(CoreError::export("GIF has zero size"));
        }
        let w16 = u16::try_from(width)
            .map_err(|_| CoreError::limit("GIF width exceeds u16 (GIF format limit)"))?;
        let h16 = u16::try_from(height)
            .map_err(|_| CoreError::limit("GIF height exceeds u16 (GIF format limit)"))?;
        // No global palette: every frame carries a local palette.
        let mut encoder = gif::Encoder::new(Vec::new(), w16, h16, &[])
            .map_err(|e| CoreError::export(format!("GIF init: {e}")))?;
        match loop_spec {
            GifLoop::None => {}
            GifLoop::Infinite => encoder
                .set_repeat(gif::Repeat::Infinite)
                .map_err(|e| CoreError::export(format!("GIF loop: {e}")))?,
            GifLoop::Finite(n) => encoder
                .set_repeat(gif::Repeat::Finite(n))
                .map_err(|e| CoreError::export(format!("GIF loop: {e}")))?,
        }
        Ok(GifWriter {
            width: w16,
            height: h16,
            encoder,
            frames: 0,
        })
    }

    pub fn frame_count(&self) -> u32 {
        self.frames
    }

    /// Quantize + append one full RGBA frame.
    pub fn add_frame(&mut self, rgba: &[u8], delay_ms: u32) -> Result<()> {
        let expected = checked_rgba_len(self.width as u32, self.height as u32, 4)?;
        if rgba.len() != expected {
            return Err(CoreError::export(format!(
                "GIF frame has {} bytes, expected {expected}",
                rgba.len()
            )));
        }
        let (w, h) = (self.width as usize, self.height as usize);
        let count = w * h;

        let mut has_transparent = false;
        for px in rgba.chunks_exact(4) {
            if px[3] < 128 {
                has_transparent = true;
                break;
            }
        }

        // Palette construction.
        let mut palette_rgb: Vec<u8>;
        let transparent_idx: Option<u8>;
        if has_transparent {
            // Train only on opaque pixels, with alpha forced to 255.
            let mut opaque: Vec<u8> = Vec::with_capacity(rgba.len());
            for px in rgba.chunks_exact(4) {
                if px[3] >= 128 {
                    opaque.extend_from_slice(&[px[0], px[1], px[2], 255]);
                }
            }
            if opaque.is_empty() {
                palette_rgb = vec![0, 0, 0];
                transparent_idx = Some(0);
                // All-transparent frame: index 0 everywhere.
                let frame = make_frame(w as u16, h as u16, vec![0u8; count], palette_rgb, transparent_idx, delay_ms)?;
                self.encoder
                    .write_frame(&frame)
                    .map_err(|e| CoreError::export(format!("GIF write: {e}")))?;
                self.frames += 1;
                return Ok(());
            }
            let nq = NeuQuant::new(10, 255, &opaque);
            palette_rgb = nq.color_map_rgb();
            debug_assert!(palette_rgb.len() <= 255 * 3);
            transparent_idx = Some((palette_rgb.len() / 3) as u8);
            palette_rgb.extend_from_slice(&[0, 0, 0]); // dummy transparent color
            debug_assert!(palette_rgb.len() <= 256 * 3);

            let mut indexed = vec![0u8; count];
            for (i, px) in rgba.chunks_exact(4).enumerate() {
                if px[3] < 128 {
                    indexed[i] = transparent_idx.unwrap();
                } else {
                    let idx = nq.index_of(&[px[0], px[1], px[2], 255]);
                    // NeuQuant may return >= 255 only if trained larger; clamp
                    // defensively to the transparent slot boundary.
                    indexed[i] = idx.min(254) as u8;
                }
            }
            let frame = make_frame(w as u16, h as u16, indexed, palette_rgb, transparent_idx, delay_ms)?;
            self.encoder
                .write_frame(&frame)
                .map_err(|e| CoreError::export(format!("GIF write: {e}")))?;
        } else {
            let nq = NeuQuant::new(10, 256, rgba);
            palette_rgb = nq.color_map_rgb();
            debug_assert!(palette_rgb.len() <= 256 * 3);
            let mut indexed = vec![0u8; count];
            for (i, px) in rgba.chunks_exact(4).enumerate() {
                let idx = nq.index_of(&[px[0], px[1], px[2], 255]);
                indexed[i] = idx.min(255) as u8;
            }
            let frame = make_frame(w as u16, h as u16, indexed, palette_rgb, None, delay_ms)?;
            self.encoder
                .write_frame(&frame)
                .map_err(|e| CoreError::export(format!("GIF write: {e}")))?;
        }
        self.frames += 1;
        Ok(())
    }

    pub fn finish(self) -> Result<Vec<u8>> {
        let bytes = self
            .encoder
            .into_inner()
            .map_err(|e| CoreError::export(format!("GIF finish: {e}")))?;
        if self.frames == 0 {
            return Err(CoreError::export("GIF contains no frames"));
        }
        Ok(bytes)
    }
}

fn make_frame(
    width: u16,
    height: u16,
    indexed: Vec<u8>,
    palette_rgb: Vec<u8>,
    transparent: Option<u8>,
    delay_ms: u32,
) -> Result<gif::Frame<'static>> {
    let delay_cs = ((delay_ms as u64 + 5) / 10).min(u16::MAX as u64) as u16;
    let mut f = gif::Frame::default();
    f.width = width;
    f.height = height;
    f.buffer = std::borrow::Cow::Owned(indexed);
    f.palette = Some(palette_rgb);
    f.transparent = transparent;
    f.delay = delay_cs;
    f.dispose = gif::DisposalMethod::Background;
    f.needs_user_input = false;
    Ok(f)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::GenericImageView;

    fn solid(w: u32, h: u32, px: [u8; 4]) -> Vec<u8> {
        let mut v = Vec::with_capacity((w * h * 4) as usize);
        for _ in 0..(w * h) {
            v.extend_from_slice(&px);
        }
        v
    }

    fn decode_all(bytes: &[u8]) -> (Vec<gif::Frame<'static>>, gif::Repeat, gif::Decoder<&[u8]>) {
        let mut opts = gif::DecodeOptions::new();
        opts.set_color_output(gif::ColorOutput::Indexed);
        let mut reader = opts.read_info(bytes).unwrap();
        let repeat = reader.repeat();
        let mut frames = Vec::new();
        loop {
            match reader.read_next_frame() {
                Ok(Some(f)) => frames.push(f.clone()),
                Ok(None) => break,
                Err(e) => panic!("decode: {e}"),
            }
        }
        (frames, repeat, reader)
    }

    #[test]
    fn still_one_frame_gif_is_valid() {
        let mut w = GifWriter::new(4, 4, GifLoop::None).unwrap();
        w.add_frame(&solid(4, 4, [10, 20, 30, 255]), 0).unwrap();
        let bytes = w.finish().unwrap();
        assert_eq!(&bytes[0..6], b"GIF89a");
        let (frames, repeat, _r) = decode_all(&bytes);
        assert_eq!(frames.len(), 1);
        // No NETSCAPE extension written for still images: decoder reports the
        // Finite(0) default == absent.
        assert!(matches!(repeat, gif::Repeat::Finite(0)));
        // Independent decode via the image crate too.
        let img = image::load_from_memory(&bytes).unwrap();
        assert_eq!(img.dimensions(), (4, 4));
    }

    #[test]
    fn animation_delays_loop_transparency_disposal() {
        let mut w = GifWriter::new(8, 8, GifLoop::Infinite).unwrap();
        // Frame 1: red, 30ms -> 3 cs.
        w.add_frame(&solid(8, 8, [255, 0, 0, 255]), 30).unwrap();
        // Frame 2: left half transparent, right half blue, 100ms -> 10 cs.
        let mut f2 = vec![0u8; 8 * 8 * 4];
        for (i, px) in f2.chunks_exact_mut(4).enumerate() {
            if i % 8 < 4 {
                px.copy_from_slice(&[0, 0, 0, 0]); // transparent
            } else {
                px.copy_from_slice(&[0, 0, 255, 255]);
            }
        }
        w.add_frame(&f2, 100).unwrap();
        let bytes = w.finish().unwrap();

        let (frames, repeat, reader) = decode_all(&bytes);
        assert_eq!(frames.len(), 2);
        assert!(matches!(repeat, gif::Repeat::Infinite));
        assert_eq!(frames[0].delay, 3);
        assert_eq!(frames[1].delay, 10);
        // Our encoder writes Background disposal (independent full frames).
        assert!(matches!(frames[0].dispose, gif::DisposalMethod::Background));
        assert!(matches!(frames[1].dispose, gif::DisposalMethod::Background));
        // Transparency preserved on frame 2.
        assert_eq!(frames[1].transparent, Some(255)); // 255-color palette -> idx 255
        let _ = reader;

        // Compose frame 2 through gif-dispose to verify actual pixels.
        let mut opts = gif::DecodeOptions::new();
        opts.set_color_output(gif::ColorOutput::Indexed);
        let mut dec = opts.read_info(bytes.as_slice()).unwrap();
        let mut screen = gif_dispose::Screen::new_decoder(&dec);
        let f1 = dec.read_next_frame().unwrap().unwrap();
        screen.blit_frame(f1).unwrap();
        {
            let px = screen.pixels_rgba();
            assert_eq!((px.rows().next().unwrap()[0].r, px.rows().next().unwrap()[0].g, px.rows().next().unwrap()[0].b), (255, 0, 0));
        }
        let f2r = dec.read_next_frame().unwrap().unwrap();
        screen.blit_frame(f2r).unwrap();
        let px = screen.pixels_rgba();
        let left = px.rows().next().unwrap()[0];
        let right = px.rows().next().unwrap()[7];
        // Left was transparent in frame2; frame1's red was disposed
        // (Background) before blitting frame2 -> transparent here.
        assert_eq!(left.a, 0, "left should be transparent after disposal");
        assert_eq!((right.r, right.g, right.b, right.a), (0, 0, 255, 255));
    }

    #[test]
    fn validates_frame_length() {
        let mut w = GifWriter::new(2, 2, GifLoop::None).unwrap();
        assert!(w.add_frame(&[0u8; 3], 0).is_err());
    }

    #[test]
    fn gif_format_dimension_limits() {
        assert!(GifWriter::new(70000, 4, GifLoop::None).is_err());
    }
}
