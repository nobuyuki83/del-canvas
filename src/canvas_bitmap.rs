pub struct Canvas {
    pub width: usize,
    pub height: usize,
    pub data: Vec<u8>,
}

impl Canvas {
    pub fn new(size: (usize, usize)) -> Self {
        Self {
            width: size.0,
            height: size.1,
            data: vec![0; size.0 * size.1 * 3],
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

    pub fn write<P: AsRef<std::path::Path>>(&mut self, path_: P) -> anyhow::Result<()> {
        // For reading and opening files
        let file = std::fs::File::create(path_)?;
        let w = std::io::BufWriter::new(file);
        let mut encoder = png::Encoder::new(w, self.width.try_into()?, self.height.try_into()?); // Width is 2 pixels and height is 1.
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(&self.data)?; // Save
        Ok(())
    }
}
