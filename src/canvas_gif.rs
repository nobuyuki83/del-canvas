
pub struct CanvasGif {
    pub width: usize,
    pub height: usize,
    pub data: Vec<u8>,
    gif_enc: Option<gif::Encoder<std::fs::File>>,
}

impl CanvasGif {
    pub fn new<Path>(path_: Path,
               size: (usize, usize),
               palette: &Vec<i32>) -> Self
        where Path: AsRef<std::path::Path>,
    {
        let res_encoder = {
            let global_palette = {
                let mut res: Vec<u8> = vec!();
                for &color in palette {
                    let (r, g, b) = crate::canvas::rgb(color);
                    res.push(r);
                    res.push(g);
                    res.push(b);
                }
                res
            };
            gif::Encoder::new(
                std::fs::File::create(path_).unwrap(),
                size.0.try_into().unwrap(),
                size.1.try_into().unwrap(),
                &global_palette)
        };
        match res_encoder {
            Err(_e) => {
                Self {
                    width: size.0,
                    height: size.1,
                    data: vec!(0; size.0 * size.1),
                    gif_enc: None,
                }
            }
            Ok(t) => {
                Self {
                    width: size.0,
                    height: size.1,
                    data: vec!(0; size.0 * size.1),
                    gif_enc: Some(t),
                }
            }
        }
    }

    pub fn clear(&mut self, color: u8) {
        for ih in 0..self.height {
            for iw in 0..self.width {
                self.data[ih * self.width + iw] = color;
            }
        }
    }

    pub fn write(&mut self) {
        // For reading and opening files
        let mut frame = gif::Frame::default();
        frame.width = self.width as u16;
        frame.height = self.height as u16;
        frame.buffer = std::borrow::Cow::Borrowed(&self.data);
        match &mut self.gif_enc {
            None => {}
            Some(enc) => {
                let _ = &enc.write_frame(&frame);
            }
        }
    }
}
