
pub struct Grid<T> {
    pub width: u32,
    pub height: u32,
    pub data: Vec<T>,
}

#[inline]
fn clamp_coord(coord: i32, min_coord: i32, max_coord: i32) -> i32 {
    if coord < min_coord {
        min_coord
    } else if coord > max_coord {
        max_coord
    } else {
        coord
    }
}

impl<T> Grid<T>
where
    T: Copy ,
{
    pub fn new(buffer: &[T], width: u32, height: u32) -> Self {
        Grid {
            width,
            height,
            data: buffer.to_vec(),
        }
    }
    pub fn get_width(&self) -> u32 {
        self.width
    }
    pub fn get_height(&self) -> u32 {
        self.height
    }
    pub fn get_buffer(&self) -> &[T] {
        &self.data
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> T {
        let idx = (y * self.width + x) as usize;
        self.data[idx]
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, value: T) {
        let idx = (y * self.width + x) as usize;
        self.data[idx] = value;
    }

    pub fn new_filled(width: u32, height: u32, value: T) -> Self {
        let len = (width as usize) * (height as usize);
        Grid {
            width,
            height,
            data: vec![value; len],
        }
    }

    pub(crate) fn get_pixel_safe(&self, x: i32, y: i32) -> T {
        // safe version of img.get_pixel() that returns 0.0 if x,y is out of bounds
        let x = x.clamp(0, self.get_width() as i32 - 1) as u32;
        let y = y.clamp(0, self.get_height() as i32 - 1) as u32;
        self.get_pixel(x, y)
    }


}


impl Grid<f32> {
    pub fn convolve(&self, kernel: &Grid<f32>) -> Grid<f32> {
        let width = self.get_width();
        let height = self.get_height();

        let kernel_width = kernel.get_width();
        let kernel_height = kernel.get_height();

        assert!(kernel_width % 2 == 1 && kernel_height % 2 == 1, "Kernel must be odd");

        let radius_x = (kernel_width / 2) as i32;
        let radius_y = (kernel_height / 2) as i32;

        let mut out = Grid::new_filled(width, height, 0_f32);

        let max_x = (width as i32) - 1;
        let max_y = (height as i32) - 1;

        for y in 0..height {
            for x in 0..width {
                let mut acc: f32 = 0.0;
                for ky in 0..kernel_height {
                    for kx in 0..kernel_width {
                        let dx = (kx as i32) - radius_x;
                        let dy = (ky as i32) - radius_y;

                        let sx = clamp_coord(x as i32 + dx, 0, max_x) as u32;
                        let sy = clamp_coord(y as i32 + dy, 0, max_y) as u32;

                        let img_v = self.get_pixel(sx, sy);
                        let k_v = kernel.get_pixel(kx, ky);
                        acc += img_v * k_v;
                    }
                }
                out.set_pixel(x, y, acc);
            }
        }
        out
    }
}


impl<T> Grid<T>
where
    T: Copy + std::ops::Sub<Output = T>,
{
    pub fn difference(&self, other: &Grid<T>) -> Grid<T> {
        assert_eq!(self.width, other.width, "width must be same");
        assert_eq!(self.height, other.height, "height must be same");

        let mut out_data = Vec::with_capacity(self.data.len());
        for (&a, &b) in self.data.iter().zip(other.data.iter()) {
            out_data.push(a - b);
        }

        Grid {
            width: self.width,
            height: self.height,
            data: out_data,
        }
    }
}
