//! Image loading: content-sniffed decoding of PNG/JPEG/WebP (stills) and
//! animated GIF, with explicit limits and EXIF orientation handling.

use crate::error::{CoreError, Result};
use crate::limits::{
    checked_rgba_len, MAX_ENCODED_BYTES, MAX_GIF_FRAMES, MAX_GIF_TOTAL_DELAY_MS,
    MAX_GIF_TOTAL_PIXELS, MAX_IMAGE_ALLOC, MAX_SOURCE_DIM, MAX_SOURCE_PIXELS,
};
use image::metadata::Orientation;
use image::{ImageReader, RgbaImage};

/// One immutable decoded RGBA frame (straight alpha, top-left origin).
#[derive(Debug, Clone)]
pub struct DecodedImage {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

impl DecodedImage {
    pub fn from_rgba(img: RgbaImage) -> Self {
        let (width, height) = img.dimensions();
        DecodedImage {
            rgba: img.into_raw(),
            width,
            height,
        }
    }
}

/// Result of loading a source file: still image or GIF animation.
#[derive(Debug, Clone)]
pub struct LoadedSource {
    /// Composited, oriented frames. Still images have exactly one frame.
    pub frames: Vec<DecodedImage>,
    /// Per-frame delay in milliseconds (len == frames.len()).
    pub delays_ms: Vec<u32>,
    /// Loop metadata: `None` = GIF carried no NETSCAPE loop extension,
    /// `Some(0)` = loop forever, `Some(n)` = loop n additional times.
    pub loop_count: Option<u32>,
    /// True when the input was a multi-frame GIF.
    pub animated: bool,
    /// Detected container name ("png" | "jpeg" | "webp" | "gif").
    pub format: &'static str,
}

fn ensure_encoded_limits(bytes: &[u8]) -> Result<()> {
    if bytes.is_empty() {
        return Err(CoreError::decode("empty file"));
    }
    if bytes.len() > MAX_ENCODED_BYTES {
        return Err(CoreError::limit(format!(
            "encoded file is {} bytes (max {})",
            bytes.len(),
            MAX_ENCODED_BYTES
        )));
    }
    Ok(())
}

/// Read EXIF orientation from the encoded bytes, when present.
/// Containers without EXIF (or files without the tag) yield `NoTransforms`.
pub fn exif_orientation(bytes: &[u8]) -> Orientation {
    // Cursor over the whole file: kamadak-exif sniffs JPEG/PNG/WebP/TIFF.
    let mut cur = std::io::Cursor::new(bytes);
    match exif::Reader::new().read_from_container(&mut cur) {
        Ok(exif) => exif
            .get_field(exif::Tag::Orientation, exif::In::PRIMARY)
            .and_then(|f| f.value.get_uint(0))
            .and_then(|v| u8::try_from(v).ok())
            .and_then(Orientation::from_exif)
            .unwrap_or(Orientation::NoTransforms),
        Err(_) => Orientation::NoTransforms,
    }
}

fn still_limits() -> image::Limits {
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(MAX_SOURCE_DIM);
    limits.max_image_height = Some(MAX_SOURCE_DIM);
    limits.max_alloc = Some(MAX_IMAGE_ALLOC);
    limits
}

/// Decode a still image (content-sniffed, EXIF-oriented).
fn decode_still(bytes: &[u8]) -> Result<DecodedImage> {
    let reader = ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| CoreError::decode(format!("format sniff failed: {e}")))?;
    let mut limits = still_limits();
    limits.max_image_width = Some(MAX_SOURCE_DIM);
    limits.max_image_height = Some(MAX_SOURCE_DIM);
    let mut reader = reader;
    reader.limits(limits);
    let mut img = reader
        .decode()
        .map_err(|e| CoreError::decode(e.to_string()))?;

    let orientation = exif_orientation(bytes);
    if orientation != Orientation::NoTransforms {
        img.apply_orientation(orientation);
    }

    let rgba = img.into_rgba8();
    let (w, h) = rgba.dimensions();
    check_source_dims(w, h)?;
    Ok(DecodedImage::from_rgba(rgba))
}

fn check_source_dims(w: u32, h: u32) -> Result<()> {
    if w == 0 || h == 0 {
        return Err(CoreError::decode("image has zero dimension"));
    }
    if w > MAX_SOURCE_DIM || h > MAX_SOURCE_DIM {
        return Err(CoreError::limit(format!(
            "image {w}x{h} exceeds max dimension {MAX_SOURCE_DIM}"
        )));
    }
    let pixels = (w as u64) * (h as u64);
    if pixels > MAX_SOURCE_PIXELS {
        return Err(CoreError::limit(format!(
            "image has {pixels} pixels (max {MAX_SOURCE_PIXELS})"
        )));
    }
    // Independent of the image crate's own allocation limits.
    checked_rgba_len(w, h, 4)?;
    Ok(())
}

/// Decode an animated GIF with correct compositing (frame rectangles,
/// transparency, disposal) plus delay and loop metadata.
fn decode_gif(bytes: &[u8]) -> Result<LoadedSource> {
    let mut opts = gif::DecodeOptions::new();
    opts.set_color_output(gif::ColorOutput::Indexed);
    // Bound decompression: indexed output is small; our pixel budget below
    // bounds the composited RGBA.
    opts.set_memory_limit(gif::MemoryLimit::Bytes(
        std::num::NonZeroU64::new(64 * 1024 * 1024).unwrap(),
    ));
    let mut decoder = opts
        .read_info(std::io::Cursor::new(bytes))
        .map_err(|e| CoreError::decode(format!("gif: {e}")))?;

    let gw = decoder.width() as u32;
    let gh = decoder.height() as u32;
    check_source_dims(gw, gh)?;

    let repeat = decoder.repeat();
    // The decoder initializes `Repeat::Finite(0)` and only overwrites it when
    // a NETSCAPE loop extension is present (count 0 -> Infinite). So
    // `Finite(0)` == "no loop extension in the file".
    let loop_count = match repeat {
        gif::Repeat::Finite(0) => None,
        gif::Repeat::Infinite => Some(0),
        gif::Repeat::Finite(n) => Some(n as u32),
    };

    let mut screen = gif_dispose::Screen::new_decoder(&decoder);
    let mut frames: Vec<DecodedImage> = Vec::new();
    let mut delays_ms: Vec<u32> = Vec::new();
    let mut total_pixels: u64 = 0;
    let mut total_delay: u64 = 0;

    loop {
        let frame = match decoder.read_next_frame() {
            Ok(Some(f)) => f,
            Ok(None) => break,
            Err(e) => return Err(CoreError::decode(format!("gif frame: {e}"))),
        };
        if frames.len() as u32 >= MAX_GIF_FRAMES {
            return Err(CoreError::limit(format!(
                "GIF has more than {MAX_GIF_FRAMES} frames"
            )));
        }
        // Delay: GIF stores 1/100s; browsers treat 0/1 special, keep as-is
        // scaled to ms (0 stays 0, we surface it to the UI).
        let delay_cs = frame.delay as u64;
        let delay_ms = delay_cs
            .checked_mul(10)
            .ok_or_else(|| CoreError::decode("gif delay overflow"))?;
        total_delay = total_delay
            .checked_add(delay_ms)
            .ok_or_else(|| CoreError::decode("gif delay overflow"))?;
        if total_delay > MAX_GIF_TOTAL_DELAY_MS {
            return Err(CoreError::limit(format!(
                "GIF total delay exceeds {} ms",
                MAX_GIF_TOTAL_DELAY_MS
            )));
        }

        screen
            .blit_frame(frame)
            .map_err(|e| CoreError::decode(format!("gif compose: {e}")))?;

        let img = screen.pixels_rgba();
        let (fw, fh) = (img.width(), img.height());
        let pixels = (fw as u64) * (fh as u64);
        total_pixels = total_pixels
            .checked_add(pixels)
            .ok_or_else(|| CoreError::limit("gif pixel budget overflow"))?;
        if total_pixels > MAX_GIF_TOTAL_PIXELS {
            return Err(CoreError::limit(format!(
                "GIF frames exceed total budget of {MAX_GIF_TOTAL_PIXELS} pixels"
            )));
        }
        let mut rgba = Vec::with_capacity((pixels * 4) as usize);
        for row in img.rows() {
            for px in row {
                rgba.extend_from_slice(&[px.r, px.g, px.b, px.a]);
            }
        }
        frames.push(DecodedImage {
            rgba,
            width: fw as u32,
            height: fh as u32,
        });
        delays_ms.push(u32::try_from(delay_ms).unwrap_or(u32::MAX));
    }

    if frames.is_empty() {
        return Err(CoreError::decode("GIF contained no frames"));
    }
    let animated = frames.len() > 1;
    Ok(LoadedSource {
        frames,
        delays_ms,
        loop_count,
        animated,
        format: "gif",
    })
}

/// Load any supported source by content sniffing (no extension dispatch).
pub fn load_source(bytes: &[u8]) -> Result<LoadedSource> {
    ensure_encoded_limits(bytes)?;

    // Sniff first so we can route GIFs to the animation path.
    let sniffed = ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| CoreError::decode(format!("format sniff failed: {e}")))?;
    let format = sniffed.format();

    match format {
        Some(image::ImageFormat::Gif) => decode_gif(bytes),
        Some(image::ImageFormat::Png)
        | Some(image::ImageFormat::Jpeg)
        | Some(image::ImageFormat::WebP) => {
            let img = decode_still(bytes)?;
            let format_name = match format {
                Some(image::ImageFormat::Png) => "png",
                Some(image::ImageFormat::Jpeg) => "jpeg",
                _ => "webp",
            };
            Ok(LoadedSource {
                frames: vec![img],
                delays_ms: vec![0],
                loop_count: None,
                animated: false,
                format: format_name,
            })
        }
        Some(other) => Err(CoreError::decode(format!(
            "unsupported image format {:?} (need PNG, JPEG, GIF or WebP)",
            other
        ))),
        None => Err(CoreError::decode(
            "unrecognized file content (need PNG, JPEG, GIF or WebP)",
        )),
    }
}
