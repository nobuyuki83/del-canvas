pub struct Canvas {
    pub width: usize,
    pub height: usize,
    pub file_path: String,
    pub tags: Vec<String>,
}

impl crate::canvas_svg::Canvas {
    pub fn new(file_path: String, size: (usize, usize)) -> Self {
        crate::canvas_svg::Canvas {
            width: size.0,
            height: size.1,
            file_path,
            tags: vec![],
        }
    }

    pub fn write(&self) {
        let mut file = std::fs::File::create(self.file_path.as_str()).expect("file not found.");
        use std::io::Write;
        writeln!(
            file,
            "<svg width=\"{}\" height=\"{}\" xmlns=\"http://www.w3.org/2000/svg\">",
            self.width, self.height
        )
        .expect("cannot write.");
        for s in &self.tags {
            writeln!(file, "{}", s).expect("cannot write");
        }
        writeln!(file, "</svg>").expect("cannot write");
    }

    pub fn polyloop(
        &mut self,
        vtx2xy: &[f32],
        transform_xy2pix: &nalgebra::Matrix3<f32>,
        stroke_color: Option<i32>,
        stroke_width: Option<f32>,
        fill: Option<i32>,
    ) {
        let s = format!(
            "<polygon points=\"{}\" {} {} {} />",
            del_msh::polyloop2::to_svg(vtx2xy, &transform_xy2pix),
            if stroke_color.is_some() {
                format!("stroke=\"#{:06X}\"", stroke_color.unwrap())
            } else {
                "stroke=\"none\"".to_owned()
            },
            if stroke_width.is_some() {
                format!("stroke-width=\"{}\"", stroke_width.unwrap())
            } else {
                "".to_owned()
            },
            if fill.is_some() {
                format!("fill=\"#{:06X}\"", fill.unwrap())
            } else {
                "fill=\"none\"".to_owned()
            }
        );
        self.tags.push(s);
    }

    pub fn circle(
        &mut self,
        x: f32,
        y: f32,
        transform_xy2pix: &nalgebra::Matrix3<f32>,
        radius: f32,
        color: &str,
    ) {
        let p = nalgebra::Vector3::<f32>::new(x, y, 1.);
        let q = transform_xy2pix * p;
        let s = format!(
            "<circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"{}\" />",
            q.x / q.z,
            q.y / q.z,
            radius,
            color
        );
        self.tags.push(s);
    }

    pub fn line(
        &mut self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        transform_xy2pix: &nalgebra::Matrix3<f32>,
        stroke_width: Option<f32>,
    ) {
        let p1 = nalgebra::Vector3::<f32>::new(x1, y1, 1.);
        let q1 = transform_xy2pix * p1;
        let p2 = nalgebra::Vector3::<f32>::new(x2, y2, 1.);
        let q2 = transform_xy2pix * p2;
        let s = format!(
            "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"black\" {} />",
            q1.x / q1.z,
            q1.y / q1.z,
            q2.x / q2.z,
            q2.y / q2.z,
            if stroke_width.is_some() {
                format!("stroke-width=\"{}\"", stroke_width.unwrap())
            } else {
                "".to_owned()
            }
        );
        self.tags.push(s);
    }
}
