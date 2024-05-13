
pub struct Canvas {
    pub width: usize,
    pub height: usize,
    data: Vec<u8>,
}

pub fn rgb(color: i32) -> (u8, u8, u8) {
    let r = ((color & 0xff0000) >> 16) as u8;
    let g = ((color & 0x00ff00) >> 8) as u8;
    let b = (color & 0x0000ff) as u8;a
    (r,g,b)
}

impl Canvas {
    pub fn new(size: (usize, usize)) -> Self {
        Self {
            width: size.0,
            height: size.1,
            data: vec!(0; size.0 * size.1 * 3),
        }
    }

    /*
    #[allow(clippy::identity_op)]
    pub fn clear(&mut self, color: i32) {
        let (r,g,b) = rgb(color);
        for ih in 0..self.height {
            for iw in 0..self.width {
                self.data[(ih * self.width + iw) * 3 + 0] = r;
                self.data[(ih * self.width + iw) * 3 + 1] = g;
                self.data[(ih * self.width + iw) * 3 + 2] = b;
            }
        }
    }
     */

    pub fn write(&mut self, path_: &std::path::Path) {
        // For reading and opening files
        let file = std::fs::File::create(path_).unwrap();
        let w = std::io::BufWriter::new(file);
        let mut encoder = png::Encoder::new(
            w,
            self.width.try_into().unwrap(),
            self.height.try_into().unwrap()); // Width is 2 pixels and height is 1.
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&self.data).unwrap(); // Save
    }
}

