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

    pub fn write<P: AsRef<std::path::Path>>(&mut self, path_: P) -> anyhow::Result<()> {
        let buffer: image::RgbImage =
            image::ImageBuffer::from_raw(self.width as u32, self.height as u32, self.data.clone())
                .unwrap();
        Ok(buffer.save(path_)?)
    }
}
