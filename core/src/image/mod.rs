//! Image loading and preprocessing

pub mod loader;
pub mod luminance;
pub mod preprocessing;

use crate::image::loader::load_image;
use crate::image::luminance::rgba_to_luma_default;
use crate::image::preprocessing::{apply_contrast, apply_gamma, resize_image};
use image::DynamicImage;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct ImageProcessor {
    rgba: Vec<u8>,
    width: u32,
    height: u32,
    luma: Vec<u8>,
}

#[wasm_bindgen]
impl ImageProcessor {
    #[wasm_bindgen(constructor)]
    pub fn new(image_bytes: &[u8]) -> Result<ImageProcessor, JsValue> {
        let img = load_image(image_bytes)?;
        let (rgba, width, height) = image_to_rgba(&img);
        let luma = rgba_to_luma_default(&rgba, width, height);
        Ok(ImageProcessor { rgba, width, height, luma })
    }

    #[wasm_bindgen]
    pub fn width(&self) -> u32 { self.width }

    #[wasm_bindgen]
    pub fn height(&self) -> u32 { self.height }

    #[wasm_bindgen]
    pub fn rgba(&self) -> Vec<u8> { self.rgba.clone() }

    #[wasm_bindgen]
    pub fn luma(&self) -> Vec<u8> { self.luma.clone() }

    #[wasm_bindgen]
    pub fn apply_preprocessing(&mut self, contrast: f32, gamma: f32) {
        apply_contrast(&mut self.rgba, contrast);
        apply_gamma(&mut self.rgba, gamma);
        // Recompute luma after preprocessing
        self.luma = rgba_to_luma_default(&self.rgba, self.width, self.height);
    }

    #[wasm_bindgen]
    pub fn resize(&mut self, target_width: u32) {
        if target_width == 0 || target_width >= self.width { return; }
        let img = DynamicImage::ImageRgba8(
            image::RgbaImage::from_raw(self.width, self.height, self.rgba.clone()).unwrap()
        );
        let resized = resize_image(&img, target_width);
        let (rgba, w, h) = image_to_rgba(&resized);
        self.rgba = rgba;
        self.width = w;
        self.height = h;
        self.luma = rgba_to_luma_default(&self.rgba, self.width, self.height);
    }
}

fn image_to_rgba(img: &DynamicImage) -> (Vec<u8>, u32, u32) {
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    (rgba.into_raw(), w, h)
}