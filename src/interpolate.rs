
pub fn calculate_resize_dimensions(width: u32, height: u32, target_long_edge: u32) -> (u32, u32) {
    let max_dimension = width.max(height);

    if max_dimension <= target_long_edge {
        return (width, height);
    }

    let scale = target_long_edge as f32 / max_dimension as f32;
    let new_width = (width as f32 * scale).round() as u32;
    let new_height = (height as f32 * scale).round() as u32;

    (new_width, new_height)
}

pub fn bilinear_resize(
    src: &[u8],
    src_width: u32,
    src_height: u32,
    dst_width: u32,
    dst_height: u32,
) -> Vec<u8> {
    let mut dst = vec![0u8; (dst_width * dst_height) as usize];

    let x_ratio = src_width as f32 / dst_width as f32;
    let y_ratio = src_height as f32 / dst_height as f32;

    for y in 0..dst_height {
        for x in 0..dst_width {
            let src_x = x as f32 * x_ratio;
            let src_y = y as f32 * y_ratio;

            let x1 = src_x.floor() as u32;
            let y1 = src_y.floor() as u32;
            let x2 = (x1 + 1).min(src_width - 1);
            let y2 = (y1 + 1).min(src_height - 1);

            let dx = src_x - x1 as f32;
            let dy = src_y - y1 as f32;

            // surrounding pixels
            let p11 = src[(y1 * src_width + x1) as usize] as f32;
            let p12 = src[(y1 * src_width + x2) as usize] as f32;
            let p21 = src[(y2 * src_width + x1) as usize] as f32;
            let p22 = src[(y2 * src_width + x2) as usize] as f32;

            // Bilinear interpolation
            let interpolated = p11 * (1.0 - dx) * (1.0 - dy) +
                p12 * dx * (1.0 - dy) +
                p21 * (1.0 - dx) * dy +
                p22 * dx * dy;

            dst[(y * dst_width + x) as usize] = interpolated.round() as u8;
        }
    }

    dst
}
