use crate::grid::Grid;

#[inline]
fn gauss(x: i32, y: i32, sigma: f32) -> f32 {
    let s2 = sigma * sigma;
    let a = 1.0 / (2.0 * std::f32::consts::PI * s2);
    let xf = x as f32;
    let yf = y as f32;
    let b = -(xf * xf + yf * yf) / (2.0 * s2);
    a * b.exp()
}

fn gaussian_kernel(size: u32, sigma: f32) -> Grid<f32> {
    assert_eq!(size % 2, 1, "size must be uneven (3, 5, 7, …)");
    assert!(sigma > 0.0, "sigma must be larger than 0.0");

    let r = (size / 2) as i32;
    let mut kernel_grid = Grid::new_filled(size, size, 0.0_f32);

    let mut sum = 0.0_f32 + f32::EPSILON;

    for dy in -r..=r {
        for dx in -r..=r {
            let v = gauss(dx, dy, sigma);
            let ix = (dx + r) as usize;
            let iy = (dy + r) as usize;
            kernel_grid.set_pixel(ix as u32, iy as u32, v);
            sum += v;
        }
    }
    for val in &mut kernel_grid.data {
        *val /= sum;
    }

    kernel_grid
}

pub fn gaussian_blur(image_grid: &Grid<f32>, kernel_size: u32, sigma: f32) -> Grid<f32> {
    let kernel = gaussian_kernel(kernel_size, sigma);
    image_grid.convolve(&kernel)
}

pub fn kernel_size_for_sigma(sigma: f32) -> u32 {
    let mut size = (sigma * 6.0).ceil() as u32;
    if size % 2 == 0 {
        size += 1;
    }
    size.max(3)
}

