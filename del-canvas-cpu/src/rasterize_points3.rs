pub trait Point2dWithColor {
    fn pix_coord(&self) -> [f32; 2];
    fn color(&self) -> [f32; 3];
}

pub fn points<POINT: Point2dWithColor>(img_size: &(usize, usize), points: &[POINT]) -> Vec<f32> {
    let mut img_data = vec![0f32; img_size.1 * img_size.0 * 3];
    for point in points.iter() {
        //let x = (point.pos_ndc[0] + 1.0) * 0.5 * (img_size.0 as f32);
        //let y = (1.0 - point.pos_ndc[1]) * 0.5 * (img_size.1 as f32);
        let scrn_pos = point.pix_coord();
        let i_x = scrn_pos[0] as usize;
        let i_y = scrn_pos[1] as usize;
        let color = point.color();
        img_data[(i_y * img_size.0 + i_x) * 3] = color[0];
        img_data[(i_y * img_size.0 + i_x) * 3 + 1] = color[1];
        img_data[(i_y * img_size.0 + i_x) * 3 + 2] = color[2];
    }
    img_data
}
