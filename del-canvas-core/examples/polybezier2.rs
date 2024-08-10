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
    let strs = del_msh_core::io_svg::svg_outline_path_from_shape(str2);
    // dbg!(&strs);
    let loops = del_msh_core::io_svg::svg_loops_from_outline_path(&strs);
    // dbg!(&loops);
    // dbg!(loops.len());
    let (width, height) = (512usize, 512usize);
    let mut img_data = vec![255u8; height * width];
    // winding number
    for i_w in 0..width {
        for i_h in 0..height {
            let p = nalgebra::Vector2::<f32>::new(i_w as f32 + 0.5f32, i_h as f32 + 0.5f32);
            let mut wn = 0.0f32;
            for i_loop in 0..loops.len() {
                wn += del_msh_core::polyloop2::winding_number(&loops[i_loop].0, &p);
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
            del_canvas_core::rasterize_line::draw_pixcenter(
                &mut img_data,
                width,
                &[p0.x, p0.y],
                &[p1.x, p1.y],
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
    let strs = del_msh_core::io_svg::svg_outline_path_from_shape(str2);
    let loops = del_msh_core::io_svg::svg_loops_from_outline_path(&strs);
    // dbg!(&loops);
    let (width, height) = (512usize, 512usize);
    let mut img_data = vec![255u8; height * width];
    for (vtx2xy, seg2vtx, is_close) in &loops {
        let vtxp2xy = del_msh_core::io_svg::polybezier2polyloop(&vtx2xy, &seg2vtx, *is_close, 0.01);
        for i_vtx in 0..vtxp2xy.len() {
            let j_vtx = (i_vtx + 1) % vtxp2xy.len();
            let p0 = vtxp2xy[i_vtx];
            let p1 = vtxp2xy[j_vtx];
            del_canvas_core::rasterize_line::draw_dda(
                &mut img_data,
                width,
                &[p0.x, p0.y],
                &[p1.x, p1.y],
                0,
            );
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

fn main() {
    hoge();
    hoge1();
}
