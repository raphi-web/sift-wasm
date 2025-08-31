pub fn rgba_to_gray(img_data: &[u8], width: usize, height: usize) -> Vec<u8> {
    let mut gray = Vec::with_capacity(width * height);
    
    for chunk in img_data.chunks_exact(4) {
        let r = chunk[0] as f32;
        let g = chunk[1] as f32;
        let b = chunk[2] as f32;
        let gray_val = (0.299 * r + 0.587 * g + 0.114 * b).round() as u8;
        gray.push(gray_val);
    }
    
    gray
}
