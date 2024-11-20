pub trait Splat2 {
    fn pos2_rgb(&self) -> ([f32; 2], [f32; 3]);
}

pub fn draw<POINT: Splat2>(img_size: &(usize, usize), points: &[POINT]) -> Vec<f32> {
    let mut img_data = vec![0f32; img_size.1 * img_size.0 * 3];
    for point in points.iter() {
        //let x = (point.pos_ndc[0] + 1.0) * 0.5 * (img_size.0 as f32);
        //let y = (1.0 - point.pos_ndc[1]) * 0.5 * (img_size.1 as f32);
        let (scrn_pos, color) = point.pos2_rgb();
        let i_x = scrn_pos[0] as usize;
        let i_y = scrn_pos[1] as usize;
        img_data[(i_y * img_size.0 + i_x) * 3] = color[0];
        img_data[(i_y * img_size.0 + i_x) * 3 + 1] = color[1];
        img_data[(i_y * img_size.0 + i_x) * 3 + 2] = color[2];
    }
    img_data
}

pub trait NdcZ {
    fn ndc_z(&self) -> f32;
}

pub fn draw_pix_sort_z<S: Splat2 + NdcZ, Path>(
    pnt2splat: &[S],
    img_shape: (usize, usize),
    path: Path,
) -> anyhow::Result<()>
where
    Path: AsRef<std::path::Path>,
{
    // draw pixels
    let idx2pnt = {
        let mut idx2pnt: Vec<usize> = (0..pnt2splat.len()).collect();
        idx2pnt.sort_by(|&idx0, &idx1| {
            let z0 = pnt2splat[idx0].ndc_z() + 1f32;
            let z1 = pnt2splat[idx1].ndc_z() + 1f32;
            z0.partial_cmp(&z1).unwrap()
        });
        idx2pnt
    };
    let mut img_data = vec![[0f32, 0f32, 0f32]; img_shape.0 * img_shape.1];
    for i_idx in 0..pnt2splat.len() {
        let i_vtx = idx2pnt[i_idx];
        let ndc_z = pnt2splat[i_vtx].ndc_z();
        if ndc_z <= -1f32 || ndc_z >= 1f32 {
            continue;
        } // clipping
        let (r0, rgb) = pnt2splat[i_vtx].pos2_rgb();
        let ix = r0[0] as usize;
        let iy = r0[1] as usize;
        let ipix = iy * img_shape.0 + ix;
        img_data[ipix][0] = rgb[0];
        img_data[ipix][1] = rgb[1];
        img_data[ipix][2] = rgb[2];
    }
    use ::slice_of_array::SliceFlatExt; // for flat
    del_canvas_image::write_png_from_float_image_rgb(path, &img_shape, (&img_data).flat())?;
    Ok(())
}
