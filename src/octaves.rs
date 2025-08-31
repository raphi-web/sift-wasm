use crate::gaussian_blur::{gaussian_blur, kernel_size_for_sigma};
use crate::grid::Grid;

fn downsample_half(src: &Grid<f32>) -> Grid<f32> {
    let new_w = (src.get_width() / 2).max(1);
    let new_h = (src.get_height() / 2).max(1);
    let mut dst_buffer = Vec::with_capacity(new_w as usize * new_h as usize);
    for y in 0..new_h {
        for x in 0..new_w {
            let sx = (x * 2).min(src.get_width() - 1);
            let sy = (y * 2).min(src.get_height() - 1);
            let v = src.get_pixel(sx, sy);
            dst_buffer.push(v);
        }
    }
    Grid {
        width: new_w,
        height: new_h,
        data: dst_buffer,
    }

}

pub fn generate_octave(
    image_grid: &Grid<f32>,
    scales: usize,
    sigma0: f32,
    sigma_n: f32,
    k: f32,
) -> (Vec<Grid<f32>>, Vec<Grid<f32>>) {
    assert!(scales >= 1, "scales must be >= 1");
    assert!(k > 1.0, "k must be > 1.0");

    let sigma_base = 0.0_f32.max(sigma0.powf(2.0) - sigma_n.powf(2.0)).sqrt();
    let mut gaussian_blurs: Vec<Grid<f32>> = Vec::with_capacity(scales + 3);

    let g0 = if sigma_base > f32::EPSILON {
        let ks = kernel_size_for_sigma(sigma_base);
        gaussian_blur(image_grid, ks, sigma_base)
    } else {
       Grid::new(image_grid.get_buffer(), image_grid.get_width(), image_grid.get_height())
    };

    gaussian_blurs.push(g0);

    // Build the remaining Gaussian levels using incremental sigma
    for i in 1..(scales + 3) {
        let sigma_prev = sigma0 * k.powi((i as i32) - 1);
        let sigma_curr = sigma0 * k.powi(i as i32);
        let sigma_inc = ((sigma_curr * sigma_curr) - (sigma_prev * sigma_prev))
            .max(0.0)
            .sqrt();

        let ks = kernel_size_for_sigma(sigma_inc);
        let prev = gaussian_blurs.last().unwrap();
        let next = gaussian_blur(prev, ks, sigma_inc);
        gaussian_blurs.push(next);
    }

    // Build DoG images: G[i+1] - G[i]
    let mut dogs: Vec<Grid<f32>> = Vec::with_capacity(scales + 2);
    for i in 0..(scales + 2) {
        let prev = &gaussian_blurs[i];
        let next = &gaussian_blurs[i + 1];
        let diff = next.difference(prev);
        dogs.push(diff);
    }

    (dogs, gaussian_blurs)
}

pub fn generate_pyramid(
    base: &Grid<f32>,
    scales: usize,
    sigma0: f32,
    sigma_n: f32,
    max_octaves: Option<usize>,
) -> (Vec<Vec<Grid<f32>>>, Vec<Vec<Grid<f32>>>) {
    assert!(scales >= 1, "scales must be >= 1");
    let k = 2.0_f32.powf(1.0 / scales as f32);

    let mut dog_vec:Vec<Vec<Grid<f32>>> = vec![];
    let mut gaussian_vec:Vec<Vec<Grid<f32>>> = vec![];

    let mut current_base = Grid::new(base.get_buffer(), base.get_width(), base.get_height());
    let mut current_sigma_n = sigma_n;

    let mut octave_idx = 0usize;
    loop {
        if let Some(limit) = max_octaves {
            if octave_idx >= limit { break; }
        }
        let min_dim = current_base.get_width().min(current_base.get_height());
        if min_dim < 16 { break; }

        let (dogs, gaussians) = generate_octave(&current_base, scales, sigma0, current_sigma_n, k);
        dog_vec.push(dogs);
        gaussian_vec.push(gaussians);

        let ref_gaussians=  gaussian_vec.last().unwrap();
        let g_s = &ref_gaussians[scales];
        current_base = downsample_half(g_s);

        current_sigma_n = sigma0;
        octave_idx += 1;
    }

    (dog_vec, gaussian_vec)
}