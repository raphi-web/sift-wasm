use crate::grid::Grid;

#[derive(Clone, Debug)]
pub struct Keypoint {
    x: f32,
    y: f32,
    octave: usize,
    level: usize,
    sigma: f32,
    angle: f32,
}

fn kernel_dxx() -> Grid<f32> {
    // [[0, 0, 0], [1, -2, 1], [0, 0, 0]]
    Grid::new(&[0.0, 0.0, 0.0, 1.0, -2.0, 1.0, 0.0, 0.0, 0.0], 3, 3)
}

fn kernel_dyy() -> Grid<f32> {
    // [[0, 1, 0], [0, -2, 0], [0, 1, 0]]
    Grid::new(&[0.0, 1.0, 0.0, 0.0, -2.0, 0.0, 0.0, 1.0, 0.0], 3, 3)
}

fn kernel_dxy() -> Grid<f32> {
    Grid::new(&[0.25, 0.0, -0.25, 0.0, 0.0, 0.0, -0.25, 0.0, 0.25], 3, 3)
}

fn hessian_terms(dog: &Grid<f32>) -> (Grid<f32>, Grid<f32>, Grid<f32>) {
    // computes the three second-order derivative images
    let dxx = dog.convolve(&kernel_dxx());
    let dyy = dog.convolve(&kernel_dyy());
    let dxy = dog.convolve(&kernel_dxy());
    (dxx, dyy, dxy)
}

fn pass_edge_response(
    dxx: &Grid<f32>,
    dyy: &Grid<f32>,
    dxy: &Grid<f32>,
    x: u32,
    y: u32,
    r: f32,
) -> bool {
    // rejects keypoints that lie on edges by checking
    // the ratio of principal curvatures of the local Hessian
    let gradient_xx = dxx.get_pixel(x, y);
    let gradient_yy = dyy.get_pixel(x, y);
    let gradient_xy = dxy.get_pixel(x, y);

    let tr = gradient_xx + gradient_yy;
    let det = gradient_xx * gradient_yy - gradient_xy * gradient_xy;
    if det <= 0.0 {
        return false;
    }
    let edge_ratio = (tr * tr) / det;
    let r_cond = ((r + 1.0) * (r + 1.0)) / r;
    edge_ratio < r_cond
}



#[inline]
fn sigma_for_level(sigma0: f32, k: f32, level: usize) -> f32 {
    // compute the scale at the given level
    sigma0 * k.powi(level as i32)
}
fn assign_orientation(gaussian: &Grid<f32>, x: u32, y: u32, sigma: f32) -> f32 {
    // This function computes the dominant orientation for a keypoint

    // Number of orientation histogram bins
    let bins = 36usize;
    // Initialize histogram array with zeros
    let mut hist = vec![0.0f32; bins];

    // Sample window radius = 3 * sigma rounded to nearest int
    let radius = (3.0 * sigma).round() as i32;
    // Gaussian weighting sigma is 1.5 times the keypoint sigma 
    let sigma_ori = 1.5 * sigma;
    // Precompute denominator of Gaussian weight formula
    let two_sigma_ori2 = 2.0 * sigma_ori * sigma_ori;

    // Convert x,y to i32 for offset calculations 
    let xc = x as i32;
    let yc = y as i32;

    // For each pixel in the sample window...
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let xx = xc + dx;
            let yy = yc + dy;

            // Skip pixels outside image bounds (with 1 pixel border)
            if xx < 1
                || yy < 1
                || xx >= gaussian.get_width() as i32 - 1
                || yy >= gaussian.get_height() as i32 - 1
            {
                continue;
            }

            // Compute gradient with central difference
            let gx = gaussian.get_pixel_safe(xx + 1, yy) - gaussian.get_pixel_safe(xx - 1, yy);
            let gy = gaussian.get_pixel_safe(xx, yy + 1) - gaussian.get_pixel_safe(xx, yy - 1);

            // Gradient magnitude 
            let mag = (gx * gx + gy * gy).sqrt();
            // Gradient orientation [-pi, pi]
            let angle = gy.atan2(gx);

            // Gaussian weight based on distance from center
            let weight = (-(dx * dx + dy * dy) as f32 / two_sigma_ori2).exp();

            // Convert angle to [0, 2pi]
            let ang = if angle < 0.0 {
                angle + std::f32::consts::TAU
            } else {
                angle
            };

            // Map angle to histogram bin (36 bins covering 360 degrees)
            let bin = ((ang * (bins as f32) / std::f32::consts::TAU).round() as usize) % bins;
            // Add weighted contribution to histogram
            hist[bin] += weight * mag;
        }
    }

    // Find bin with highest peak
    let (mut max_bin, mut max_val) = (0usize, -1.0f32);
    for (i, &v) in hist.iter().enumerate() {
        if v > max_val {
            max_val = v;
            max_bin = i;
        }
    }

    // Convert max bin index to angle in radians
    (max_bin as f32 + 0.5) * (std::f32::consts::TAU / bins as f32)
}
fn is_local_extremum(
    dogs_octave: &[Grid<f32>],
    s: usize,
    x: u32,
    y: u32,
    contrast_thresh: f32,
) -> bool {
    // Get pixel value at current location
    let center = dogs_octave[s].get_pixel(x, y);

    // First check if pixel value exceeds minimum contrast threshold
    if center.abs() < contrast_thresh {
        return false;
    }

    // Track if center is greater/less than ALL neighbors
    let mut greater = true;  // Center > all neighbors
    let mut less = true;     // Center < all neighbors

    // Convert x,y to i32 for offset calculations
    let xi = x as i32;
    let yi = y as i32;

    // Check 26 neighbors (3x3x3 cube excluding center)
    for ds in (s - 1)..=(s + 1) {         // Scale dimension
        let img = &dogs_octave[ds];
        for dy in -1..=1 {                 // Y dimension  
            for dx in -1..=1 {             // X dimension
                // Skip comparison with center pixel itself
                if ds == s && dx == 0 && dy == 0 {
                    continue;
                }

                // Get neighbor pixel value
                let v = img.get_pixel_safe(xi + dx, yi + dy);

                // Update greater/less flags based on comparison
                if center <= v {
                    greater = false;  // Not greater than ALL neighbors
                }
                if center >= v {
                    less = false;    // Not less than ALL neighbors
                }

                // Early exit if center is neither max nor min
                if !(greater || less) {
                    return false;
                }
            }
        }
    }

    // Return true if center was either greater or less than ALL neighbors
    true
}
pub fn detect_keypoints(
    dogs: &[Vec<Grid<f32>>],
    gaussians: &[Vec<Grid<f32>>],
    scales: usize,
    sigma0: f32,
    contrast_thresh: f32,
    edge_r: f32,
    k: f32,
) -> Vec<Keypoint> {
    // Vector to store detected keypoints
    let mut keypoints = Vec::new();

    // Iterate through each octave in the DoG pyramid
    for (octave, dogs_octave) in dogs.iter().enumerate() {
        // Process each scale level except first and last
        for scale_level in 1..=scales {
            let difference_of_gaussian = &dogs_octave[scale_level];
            let image_width = difference_of_gaussian.get_width();
            let image_height = difference_of_gaussian.get_height();

            // Skip if image is too small
            if image_width < 3 || image_height < 3 {
                continue;
            }

            // Build Hessian maps once for this level for edge response testing
            let (hessian_xx, hessian_yy, hessian_xy) = hessian_terms(difference_of_gaussian);

            // Scan interior pixels of the image (exclude borders)
            for y_coord in 1..(image_height - 1) {
                for x_coord in 1..(image_width - 1) {
                    // Check if point is local extremum with sufficient contrast
                    if !is_local_extremum(dogs_octave, scale_level, x_coord, y_coord, contrast_thresh) {
                        continue;
                    }

                    // Check if point passes edge response test
                    if !pass_edge_response(&hessian_xx, &hessian_yy, &hessian_xy, x_coord, y_coord, edge_r) {
                        continue;
                    }

                    // Calculate scale and orientation for valid keypoint
                    let keypoint_sigma = sigma_for_level(sigma0, k, scale_level);
                    let keypoint_angle = assign_orientation(&gaussians[octave][scale_level], x_coord, y_coord, keypoint_sigma);

                    // Store the keypoint
                    keypoints.push(Keypoint {
                        x: x_coord as f32,
                        y: y_coord as f32,
                        octave: octave,
                        level: scale_level,
                        sigma: keypoint_sigma,
                        angle: keypoint_angle,
                    });
                }
            }
        }
    }
    keypoints
}
pub fn flatten_keypoints(kps: &[Keypoint]) -> Vec<f32> {
    // flattens keypoint to a vector of 6 floats: x, y, octave, level, sigma, angle
    // for return to js
    let mut out = Vec::with_capacity(kps.len() * 6);
    for kp in kps {
        out.push(kp.x);
        out.push(kp.y);
        out.push(kp.octave as f32);
        out.push(kp.level as f32);
        out.push(kp.sigma);
        out.push(kp.angle);
    }
    out
}

fn wrap_angle_2pi(a: f32) -> f32 {
    let mut ang = a % std::f32::consts::TAU;
    if ang < 0.0 {
        ang += std::f32::consts::TAU;
    }
    ang
}

// Compute a 128-D SIFT descriptor for one keypoint
fn compute_descriptor_for(
    gaussian: &Grid<f32>,
    kp_x: f32,
    kp_y: f32,
    kp_sigma: f32,
    kp_angle: f32,
) -> [f32; 128] {
    // The SIFT descriptor creates a 128-dimensional vector that describes the local image region
    // It divides the region into a 4x4 grid of cells, with 8 orientation bins per cell (4x4x8 = 128 total values)
    const N_CELLS: usize = 4; // 4x4 spatial cells to sample
    const N_BINS: usize = 8;  // 8 orientation bins per cell
    const DESC_LEN: usize = N_CELLS * N_CELLS * N_BINS; // 128 total descriptor values

    // Get rotation values to align descriptor with keypoint orientation
    let cos_t = kp_angle.cos();
    let sin_t = kp_angle.sin();

    // Each spatial cell covers 4 pixels at the keypoint's scale 
    let bin_size: f32 = 4.0;
    // Gaussian weighting sigma controls falloff of samples away from cell center
    let sigma_descr: f32 = 0.5 * (N_CELLS as f32);

    // Calculate sample window radius in pixels to roughly cover all 4x4 cells
    let radius: i32 = (kp_sigma * bin_size * (N_CELLS as f32) * 0.5 * 1.41421356).ceil() as i32;

    // Initialize 3D histogram array [4x4x8] to accumulate orientation samples
    let mut hist = [0.0f32; DESC_LEN];

    let width = gaussian.get_width() as i32;
    let height = gaussian.get_height() as i32;
    let keypoint_x = kp_x as i32;
    let keypoint_y = kp_y as i32;

    // Sample pixels in window around keypoint
    for delta_y in -radius..=radius {
        for delta_x in -radius..=radius {
            let image_x = keypoint_x + delta_x;
            let image_y = keypoint_y + delta_y;

            // Skip pixels outside image boundaries
            if image_x <= 0 || image_y <= 0 || image_x >= width - 1 || image_y >= height - 1 {
                continue;
            }

            // Calculate gradient magnitude and direction using central differences
            let gradient_x = gaussian.get_pixel_safe(image_x + 1, image_y) - gaussian.get_pixel_safe(image_x - 1, image_y);
            let gradient_y = gaussian.get_pixel_safe(image_x, image_y + 1) - gaussian.get_pixel_safe(image_x, image_y - 1);
            let magnitude = (gradient_x * gradient_x + gradient_y * gradient_y).sqrt();
            if magnitude == 0.0 {
                continue;
            }

            // Rotate and scale coordinates relative to keypoint orientation and scale
            let delta_x_float = delta_x as f32;
            let delta_y_float = delta_y as f32;
            let rotated_x = (cos_t * delta_x_float + sin_t * delta_y_float) / bin_size / kp_sigma;
            let rotated_y = (-sin_t * delta_x_float + cos_t * delta_y_float) / bin_size / kp_sigma;

            // Convert to cell coordinates with offset for interpolation
            let cell_x = rotated_x + (N_CELLS as f32) / 2.0 - 0.5;
            let cell_y = rotated_y + (N_CELLS as f32) / 2.0 - 0.5;

            // Skip samples outside descriptor cells
            if cell_x <= -1.0 || cell_y <= -1.0 || cell_x >= N_CELLS as f32 || cell_y >= N_CELLS as f32 {
                continue;
            }

            // Calculate orientation bin value relative to keypoint angle
            let angle = wrap_angle_2pi(gradient_y.atan2(gradient_x) - kp_angle);
            let orientation_bin = angle * (N_BINS as f32) / std::f32::consts::TAU;

            // Calculate Gaussian weight based on distance from keypoint
            let weight = (-(rotated_x * rotated_x + rotated_y * rotated_y) / (2.0 * sigma_descr * sigma_descr)).exp();

            // Get base indices and fractions for trilinear interpolation
            let index_x0 = cell_x.floor() as i32;
            let index_y0 = cell_y.floor() as i32;
            let index_orientation0 = orientation_bin.floor() as i32;

            let fraction_x = cell_x - index_x0 as f32;
            let fraction_y = cell_y - index_y0 as f32;
            let fraction_orientation = orientation_bin - index_orientation0 as f32;

            // Trilinear interpolation across neighboring cells and bins
            for (delta_x_index, weight_x) in [(0, 1.0 - fraction_x), (1, fraction_x)] {
                let index_x_bin = index_x0 + delta_x_index;
                if index_x_bin < 0 || index_x_bin >= N_CELLS as i32 {
                    continue;
                }
                for (delta_y_index, weight_y) in [(0, 1.0 - fraction_y), (1, fraction_y)] {
                    let index_y_bin = index_y0 + delta_y_index;
                    if index_y_bin < 0 || index_y_bin >= N_CELLS as i32 {
                        continue;
                    }
                    for (delta_orientation_index, weight_orientation) in [(0, 1.0 - fraction_orientation), (1, fraction_orientation)] {
                        let mut index_orientation_bin = index_orientation0 + delta_orientation_index;
                        // Wrap orientation bin around at boundaries
                        while index_orientation_bin < 0 {
                            index_orientation_bin += N_BINS as i32;
                        }
                        while index_orientation_bin >= N_BINS as i32 {
                            index_orientation_bin -= N_BINS as i32;
                        }

                        // Calculate final weight and add to histogram
                        let final_weight = weight * magnitude * weight_x * weight_y * weight_orientation;
                        let index = ((index_y_bin as usize) * N_CELLS + (index_x_bin as usize)) * N_BINS
                            + (index_orientation_bin as usize);
                        hist[index] += final_weight;
                    }
                }
            }
        }
    }

    // Normalize descriptor vector for illumination invariance
    // 1) First L2 normalization
    let mut norm = 0.0f32;
    for v in &hist {
        norm += v * v;
    }
    norm = norm.sqrt();
    if norm > 0.0 {
        for v in &mut hist.iter_mut() {
            *v /= norm;
        }
    }
    // 2) Threshold values at 0.2 to reduce influence of large gradients
    for v in &mut hist.iter_mut() {
        if *v > 0.2 {
            *v = 0.2;
        }
    }
    // 3) Renormalize again
    norm = 0.0;
    for v in &hist {
        norm += v * v;
    }
    norm = norm.sqrt();
    if norm > 0.0 {
        for v in &mut hist.iter_mut() {
            *v /= norm;
        }
    }

    hist
}
// Public: Extract 128D descriptors for a list of keypoints.
// Returns a flat Vec<f32> of length 128 * keypoints.len(), in the same order.
pub fn extract_descriptors(gaussians: &[Vec<Grid<f32>>], keypoints: &[Keypoint]) -> Vec<f32> {
    let mut out = Vec::with_capacity(keypoints.len() * 128);
    for kp in keypoints {
        // Use the Gaussian image at the keypoint’s octave/level
        let g = &gaussians[kp.octave][kp.level];
        let desc = compute_descriptor_for(g, kp.x, kp.y, kp.sigma, kp.angle);
        out.extend_from_slice(&desc);
    }
    out
}
