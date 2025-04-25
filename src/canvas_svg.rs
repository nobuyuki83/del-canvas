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
        transform_xy2pix: &[f32; 9],
        stroke_color: Option<i32>,
        stroke_width: Option<f32>,
        fill: Option<i32>,
    ) {
        let s = format!(
            "<polygon points=\"{}\" {} {} {} />",
            polyloop2_to_svg(vtx2xy, transform_xy2pix),
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
        transform_xy2pix: &[f32; 9],
        radius: f32,
        color: &str,
    ) {
        let p = [x, y, 1.];
        let q = del_geo_core::mat3_col_major::mult_vec(transform_xy2pix, &p);
        let s = format!(
            "<circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"{}\" />",
            q[0] / q[2],
            q[1] / q[2],
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
        transform_xy2pix: &[f32; 9],
        stroke_width: Option<f32>,
    ) {
        let p1 = [x1, y1, 1.];
        let q1 = del_geo_core::mat3_col_major::mult_vec(transform_xy2pix, &p1);
        let p2 = [x2, y2, 1.];
        let q2 = del_geo_core::mat3_col_major::mult_vec(transform_xy2pix, &p2);
        let s = format!(
            "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"black\" {} />",
            q1[0] / q1[2],
            q1[1] / q1[2],
            q2[0] / q2[2],
            q2[1] / q2[2],
            if stroke_width.is_some() {
                format!("stroke-width=\"{}\"", stroke_width.unwrap())
            } else {
                "".to_owned()
            }
        );
        self.tags.push(s);
    }
}

#[test]
fn hoge() {
    let str2 = "M 457.60409,474.77081 H 347.66161 \
    L 208.25942,282.21963 q -15.48914,0.60741 -25.20781,0.60741 \
    -3.94821,0 -8.50384,0 -4.55562,-0.3037 -9.41496,-0.60741 \
    v 119.66114 q 0,38.87469 8.50384,48.28965 11.54092,13.36318 34.62277,13.36318 \
    h 16.09655 v 11.23721 H 47.901331 V 463.5336 h 15.489133 \
    q 26.118931,0 37.356146,-17.00768 6.37788,-9.41496 6.37788,-44.64515 V 135.83213 \
    q 0,-38.874683 -8.50384,-48.289646 Q 86.776018,74.17931 63.390464,74.17931 H 47.901331 \
    V 62.942096 H 197.93333 q 65.60103,0 96.5793,9.718671 31.28197,9.414964 52.84528,35.230183 \
    21.86701,25.51152 21.86701,61.04541 0,37.96356 -24.9041,65.90474 -24.60039,27.94118 -76.53454,39.48211 \
    l 85.03838,118.1426 q 29.15601,40.69694 50.1119,54.06011 20.95589,13.36318 54.66753,17.00768 z \
    M 165.13281,263.08599 q 5.77046,0 10.02238,0.30371 4.25192,0 6.9853,0 58.91944,0 88.68288,-25.51151 \
    30.06714,-25.51152 30.06714,-64.99362 0,-38.57098 -24.29668,-62.56395 -23.99297,-24.296679 \
    -63.77879,-24.296679 -17.61509,0 -47.68223,5.770461 z";
    let strs = svg_outline_path_from_shape(str2);
    // dbg!(&strs);
    let loops = svg_loops_from_outline_path(&strs);
    // dbg!(&loops);
    // dbg!(loops.len());
    let (width, height) = (512usize, 512usize);
    let mut img_data = vec![255u8; height * width];
    // winding number
    for i_w in 0..width {
        for i_h in 0..height {
            let p = [i_w as f32 + 0.5f32, i_h as f32 + 0.5f32];
            let mut wn = 0.0f32;
            for i_loop in 0..loops.len() {
                use slice_of_array::SliceFlatExt;
                let loop0 = loops[i_loop].0.flat();
                wn += crate::rasterize::polygon2::winding_number(loop0, &p);
            }
            if wn.round() as i64 != 0 {
                img_data[i_h * width + i_w] = 128;
            }
        }
    }
    // dda
    for (vtx2xy, _seg2vtx, _is_close) in &loops {
        let num_vtx = vtx2xy.len();
        for i_vtx in 0..num_vtx {
            let j_vtx = (i_vtx + 1) % num_vtx;
            let p0 = &vtx2xy[i_vtx];
            let p1 = &vtx2xy[j_vtx];
            crate::rasterize::line2::draw_pixcenter(
                &mut img_data,
                width,
                p0,
                p1,
                &[1., 0., 0., 0., 1., 0., 0., 0., 1.],
                3.0,
                0,
            );
        }
    }
    let file = std::fs::File::create("target/r0.png").unwrap();
    let w = std::io::BufWriter::new(file);
    let mut encoder = png::Encoder::new(w, width.try_into().unwrap(), height.try_into().unwrap()); // Width is 2 pixels and height is 1.
    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(&img_data).unwrap(); // Save
}

#[test]
fn hoge1() {
    let str2 = "M 457.60409,474.77081 H 347.66161 \
    L 208.25942,282.21963 q -15.48914,0.60741 -25.20781,0.60741 \
    -3.94821,0 -8.50384,0 -4.55562,-0.3037 -9.41496,-0.60741 \
    v 119.66114 q 0,38.87469 8.50384,48.28965 11.54092,13.36318 34.62277,13.36318 \
    h 16.09655 v 11.23721 H 47.901331 V 463.5336 h 15.489133 \
    q 26.118931,0 37.356146,-17.00768 6.37788,-9.41496 6.37788,-44.64515 V 135.83213 \
    q 0,-38.874683 -8.50384,-48.289646 Q 86.776018,74.17931 63.390464,74.17931 H 47.901331 \
    V 62.942096 H 197.93333 q 65.60103,0 96.5793,9.718671 31.28197,9.414964 52.84528,35.230183 \
    21.86701,25.51152 21.86701,61.04541 0,37.96356 -24.9041,65.90474 -24.60039,27.94118 -76.53454,39.48211 \
    l 85.03838,118.1426 q 29.15601,40.69694 50.1119,54.06011 20.95589,13.36318 54.66753,17.00768 z \
    M 165.13281,263.08599 q 5.77046,0 10.02238,0.30371 4.25192,0 6.9853,0 58.91944,0 88.68288,-25.51151 \
    30.06714,-25.51152 30.06714,-64.99362 0,-38.57098 -24.29668,-62.56395 -23.99297,-24.296679 \
    -63.77879,-24.296679 -17.61509,0 -47.68223,5.770461 z";
    let strs = svg_outline_path_from_shape(str2);
    let loops = svg_loops_from_outline_path(&strs);
    // dbg!(&loops);
    let (width, height) = (512usize, 512usize);
    let mut img_data = vec![255u8; height * width];
    for (vtx2xy, seg2vtx, is_close) in &loops {
        let vtxp2xy = polybezier2polyloop(&vtx2xy, &seg2vtx, *is_close, 0.01);
        for i_vtx in 0..vtxp2xy.len() {
            let j_vtx = (i_vtx + 1) % vtxp2xy.len();
            let p0 = vtxp2xy[i_vtx];
            let p1 = vtxp2xy[j_vtx];
            crate::rasterize::line2::draw_dda_pixel_coordinate(&mut img_data, width, &p0, &p1, 0);
        }
    }
    let file = std::fs::File::create("target/r1.png").unwrap();
    let w = std::io::BufWriter::new(file);
    let mut encoder = png::Encoder::new(w, width.try_into().unwrap(), height.try_into().unwrap()); // Width is 2 pixels and height is 1.
    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(&img_data).unwrap(); // Save
}

pub fn polyloop2_to_svg<Real>(vtx2xy: &[Real], transform: &[Real; 9]) -> String
where
    Real: std::fmt::Display + Copy + num_traits::Float,
{
    let mut res = String::new();
    for ivtx in 0..vtx2xy.len() / 2 {
        let x = vtx2xy[ivtx * 2];
        let y = vtx2xy[ivtx * 2 + 1];
        let a = del_geo_core::mat3_col_major::transform_homogeneous(transform, &[x, y]).unwrap();
        res += format!("{} {}", a[0], a[1]).as_str();
        if ivtx != vtx2xy.len() / 2 - 1 {
            res += ",";
        }
    }
    res
}

pub fn svg_outline_path_from_shape(s0: &str) -> Vec<String> {
    let s0 = s0.as_bytes();
    let mut imark = 0;
    let mut strs = Vec::<String>::new();
    for i in 0..s0.len() {
        if s0[i].is_ascii_digit() {
            continue;
        }
        if s0[i] == b',' {
            strs.push(std::str::from_utf8(&s0[imark..i]).unwrap().to_string());
            imark = i + 1; // mark should be the beginning position of the string so move next
        }
        if s0[i] == b' ' {
            // sometimes the space act as delimiter in the SVG (inkscape version)
            if i > imark {
                strs.push(std::str::from_utf8(&s0[imark..i]).unwrap().to_string());
            }
            imark = i + 1; // mark should be the beginning position of the string so move next
        }
        if s0[i] == b'-' {
            if i > imark {
                strs.push(std::str::from_utf8(&s0[imark..i]).unwrap().to_string());
            }
            imark = i;
        }
        if s0[i].is_ascii_alphabetic() {
            if i > imark {
                strs.push(std::str::from_utf8(&s0[imark..i]).unwrap().to_string());
            }
            strs.push(std::str::from_utf8(&[s0[i]]).unwrap().to_string()); // push tag
            imark = i + 1;
        }
    }
    if s0.len() > imark {
        strs.push(
            std::str::from_utf8(&s0[imark..s0.len()])
                .unwrap()
                .to_string(),
        );
    }
    strs
}

#[allow(clippy::identity_op)]
pub fn svg_loops_from_outline_path(strs: &[String]) -> Vec<(Vec<[f32; 2]>, Vec<usize>, bool)> {
    use del_geo_core::vec2::Vec2;
    let hoge = |s0: &str, s1: &str| [s0.parse::<f32>().unwrap(), s1.parse::<f32>().unwrap()];
    let mut loops: Vec<(Vec<[f32; 2]>, Vec<usize>, bool)> = vec![];
    let mut vtxl2xy: Vec<[f32; 2]> = vec![];
    let mut seg2vtxl: Vec<usize> = vec![0];
    assert!(strs[0] == "M" || strs[0] == "m");
    // assert!(strs[strs.len() - 1] == "Z" || strs[strs.len() - 1] == "z");
    let mut pos_cur = [0f32; 2];
    let mut is = 0;
    loop {
        if strs[is] == "M" {
            // start absolute
            is += 1;
            pos_cur = hoge(&strs[is + 0], &strs[is + 1]);
            vtxl2xy.push(pos_cur);
            is += 2;
        } else if strs[is] == "m" {
            // start relative
            is += 1;
            pos_cur = pos_cur.add(&hoge(&strs[is + 0], &strs[is + 1]));
            vtxl2xy.push(pos_cur);
            is += 2;
        } else if strs[is] == "l" {
            // line relative
            is += 1;
            loop {
                seg2vtxl.push(vtxl2xy.len());
                let p1 = pos_cur.add(&hoge(&strs[is + 0], &strs[is + 1]));
                vtxl2xy.push(p1);
                pos_cur = p1;
                is += 2;
                if strs[is].as_bytes()[0].is_ascii_alphabetic() {
                    break;
                }
            }
        } else if strs[is] == "L" {
            // line absolute
            is += 1;
            loop {
                seg2vtxl.push(vtxl2xy.len());
                let p1 = hoge(&strs[is + 0], &strs[is + 1]);
                vtxl2xy.push(p1);
                pos_cur = p1;
                is += 2;
                if strs[is].as_bytes()[0].is_ascii_alphabetic() {
                    break;
                }
            }
        } else if strs[is] == "v" {
            // vertical relative
            seg2vtxl.push(vtxl2xy.len());
            let p1 = pos_cur.add(&[0., strs[is + 1].parse::<f32>().unwrap()]);
            vtxl2xy.push(p1);
            pos_cur = p1;
            is += 2;
        } else if strs[is] == "V" {
            // vertical absolute
            seg2vtxl.push(vtxl2xy.len());
            let p1 = [pos_cur[0], strs[is + 1].parse::<f32>().unwrap()];
            vtxl2xy.push(p1);
            pos_cur = p1;
            is += 2;
        } else if strs[is] == "H" {
            // horizontal absolute
            seg2vtxl.push(vtxl2xy.len());
            let p1 = [strs[is + 1].parse::<f32>().unwrap(), pos_cur[1]];
            vtxl2xy.push(p1);
            pos_cur = p1;
            is += 2;
        } else if strs[is] == "h" {
            // horizontal relative
            seg2vtxl.push(vtxl2xy.len());
            let dh = strs[is + 1].parse::<f32>().unwrap();
            let p1 = pos_cur.add(&[dh, 0.]);
            vtxl2xy.push(p1);
            pos_cur = p1;
            is += 2;
        } else if strs[is] == "q" {
            // relative
            is += 1;
            loop {
                // loop for poly quadratic Bézeir curve
                let pm0 = pos_cur.add(&hoge(&strs[is + 0], &strs[is + 1]));
                let p1 = pos_cur.add(&hoge(&strs[is + 2], &strs[is + 3]));
                vtxl2xy.push(pm0);
                seg2vtxl.push(vtxl2xy.len());
                vtxl2xy.push(p1);
                pos_cur = p1;
                is += 4;
                if is == strs.len() {
                    loops.push((vtxl2xy.clone(), seg2vtxl.clone(), false));
                    break;
                }
                if strs[is].as_bytes()[0].is_ascii_alphabetic() {
                    break;
                }
            }
        } else if strs[is] == "Q" {
            // absolute
            is += 1;
            loop {
                // loop for poly-Bezeir curve
                let pm0 = hoge(&strs[is + 0], &strs[is + 1]);
                let p1 = hoge(&strs[is + 2], &strs[is + 3]);
                vtxl2xy.push(pm0);
                seg2vtxl.push(vtxl2xy.len());
                vtxl2xy.push(p1);
                pos_cur = p1;
                is += 4;
                if strs[is].as_bytes()[0].is_ascii_alphabetic() {
                    break;
                }
            }
        } else if strs[is] == "c" {
            // relative
            is += 1;
            loop {
                // loop for poly cubic Bézeir curve
                let pm0 = pos_cur.add(&hoge(&strs[is + 0], &strs[is + 1]));
                let pm1 = pos_cur.add(&hoge(&strs[is + 2], &strs[is + 3]));
                let p1 = pos_cur.add(&hoge(&strs[is + 4], &strs[is + 5]));
                vtxl2xy.push(pm0);
                vtxl2xy.push(pm1);
                seg2vtxl.push(vtxl2xy.len());
                vtxl2xy.push(p1);
                pos_cur = p1;
                is += 6;
                if is == strs.len() {
                    loops.push((vtxl2xy.clone(), seg2vtxl.clone(), false));
                    break;
                }
                if strs[is].as_bytes()[0].is_ascii_alphabetic() {
                    break;
                }
            }
        } else if strs[is] == "z" || strs[is] == "Z" {
            let pe = vtxl2xy[0];
            let ps = vtxl2xy[vtxl2xy.len() - 1];
            let _dist0 = ps.sub(&pe).norm();
            loops.push((vtxl2xy.clone(), seg2vtxl.clone(), true));
            vtxl2xy.clear();
            seg2vtxl = vec![0];
            is += 1;
        } else {
            dbg!("error!--> ", &strs[is]);
            break;
        }
        if is == strs.len() {
            break;
        }
    }
    loops
}
pub fn polybezier2polyloop(
    vtx2xy: &[[f32; 2]],
    seg2vtx: &[usize],
    is_close: bool,
    edge_length: f32,
) -> Vec<[f32; 2]> {
    use del_geo_core::vec2::Vec2;
    let mut ret: Vec<[f32; 2]> = vec![];
    let num_seg = seg2vtx.len() - 1;
    for i_seg in 0..num_seg {
        let (is_vtx, ie_vtx) = (seg2vtx[i_seg], seg2vtx[i_seg + 1]);
        let ps = &vtx2xy[is_vtx];
        let pe = &vtx2xy[ie_vtx];
        if ie_vtx - is_vtx == 1 {
            let len = pe.sub(ps).norm();
            let ndiv = (len / edge_length) as usize;
            for i in 0..ndiv {
                let r = i as f32 / ndiv as f32;
                let p = ps.scale(1f32 - r).add(&pe.scale(r));
                ret.push(p);
            }
        } else if ie_vtx - is_vtx == 2 {
            // quadratic bezier
            let pc = &vtx2xy[is_vtx + 1];
            let ndiv = 10;
            for idiv in 0..ndiv {
                let t0 = idiv as f32 / ndiv as f32;
                let p0 = del_geo_core::bezier_quadratic::eval(ps, pc, pe, t0);
                ret.push(p0);
            }
        } else if ie_vtx - is_vtx == 3 {
            // cubic bezier
            let pc0 = &vtx2xy[is_vtx + 1];
            let pc1 = &vtx2xy[is_vtx + 2];
            let samples = del_geo_core::bezier_cubic::sample_uniform_length(
                del_geo_core::bezier_cubic::ControlPoints::<'_, f32, 2> {
                    p0: ps,
                    p1: pc0,
                    p2: pc1,
                    p3: pe,
                },
                edge_length,
                true,
                false,
                30,
            );
            ret.extend(samples);
        }
    }
    if is_close {
        let ps = &vtx2xy[vtx2xy.len() - 1];
        let pe = &vtx2xy[0];
        let len = pe.sub(ps).norm();
        let ndiv = (len / edge_length) as usize;
        for i in 0..ndiv {
            let r = i as f32 / ndiv as f32;
            let p = ps.scale(1f32 - r).add(&pe.scale(r));
            ret.push(p);
        }
    }
    ret
}
