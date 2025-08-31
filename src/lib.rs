mod gaussian_blur;
mod grid;
mod interpolate;
mod keypoints;
mod match_keypoints;
mod octaves;
mod rgb_to_gray;

use crate::grid::Grid;
use crate::interpolate::{bilinear_resize, calculate_resize_dimensions};
use crate::keypoints::{detect_keypoints, extract_descriptors, flatten_keypoints};
use crate::octaves::generate_pyramid;
use wasm_bindgen::prelude::*;
// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub struct ResizeResult {
    data: Vec<u8>,
    width: u32,
    height: u32,
}

#[wasm_bindgen]
impl ResizeResult {
    #[wasm_bindgen(getter)]
    pub fn data(&self) -> Vec<u8> {
        self.data.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn width(&self) -> u32 {
        self.width
    }
    #[wasm_bindgen(getter)]
    pub fn height(&self) -> u32 {
        self.height
    }
}

#[wasm_bindgen]
pub fn resize_image(
    image_buffer: &[u8],
    original_width: u32,
    original_height: u32,
    target_long_edge: u32,
) -> ResizeResult {
    let (new_width, new_height) =
        calculate_resize_dimensions(original_width, original_height, target_long_edge);

    let resized_data = bilinear_resize(
        image_buffer,
        original_width,
        original_height,
        new_width,
        new_height,
    );

    ResizeResult {
        data: resized_data,
        width: new_width,
        height: new_height,
    }
}

#[wasm_bindgen]
pub fn rgba_to_gray(image_buffer: &[u8], width: u32, height: u32) -> Vec<u8> {
    rgb_to_gray::rgba_to_gray(image_buffer, width as usize, height as usize)
}

#[wasm_bindgen]
pub struct SiftResult {
    keypoints: Vec<f32>,   // [x, y, octave, level, sigma, angle, ...]
    descriptors: Vec<f32>, // 128D per keypoint
}

#[wasm_bindgen]
impl SiftResult {
    #[wasm_bindgen(getter)]
    pub fn keypoints(&self) -> Vec<f32> {
        self.keypoints.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn descriptors(&self) -> Vec<f32> {
        self.descriptors.clone()
    }
}

#[wasm_bindgen]
pub fn sift(image_buffer: &[u8], width: u32, height: u32, scales: usize) -> SiftResult {
    // Convert input to f32
    let base_data: Vec<f32> = image_buffer.iter().map(|&v| v as f32).collect();
    let base = Grid::new(&base_data, width, height);

    // Parameters
    let sigma0 = 1.6_f32;
    let sigma_n = 0.5_f32;
    let contrast_thresh = 0.03_f32;
    let edge_r = 10.0_f32;
    let k = 2.0_f32.powf(1.0 / scales as f32);

    // Build pyramid
    let (dogs, gaussians) = generate_pyramid(&base, scales, sigma0, sigma_n, None);

    // Detect keypoints
    let kps = detect_keypoints(
        &dogs,
        &gaussians,
        scales,
        sigma0,
        contrast_thresh,
        edge_r,
        k,
    );

    // Flat keypoints for JS
    let kps_flat = flatten_keypoints(&kps);

    // Descriptors aligned with kps
    let desc = extract_descriptors(&gaussians, &kps);

    SiftResult {
        keypoints: kps_flat,
        descriptors: desc,
    }
}

#[wasm_bindgen]
pub fn match_descriptors_topk(
    desc1: &[f32],
    desc2: &[f32],
    d: usize,
    ratio: f32,
    cross_check: bool,
    top_k: usize,
) -> Vec<u32> {
    crate::match_keypoints::match_descriptors_topk_impl(desc1, desc2, d, ratio, cross_check, top_k)
}
