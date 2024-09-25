pub trait Splat2 {
    fn ndc_z(&self) -> f32;
    fn property(&self) -> (&[f32; 2], &[f32; 3]);
}

pub fn draw_pix_sort_z<S: Splat2, Path>(
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
            let z0 = pnt2splat[idx0].ndc_z();
            let z1 = pnt2splat[idx1].ndc_z();
            z0.partial_cmp(&z1).unwrap()
        });
        idx2pnt
    };
    let mut img_data = vec![[0f32, 0f32, 0f32]; img_shape.0 * img_shape.1];
    for i_idx in 0..pnt2splat.len() {
        let i_vtx = idx2pnt[i_idx];
        let (r0, rgb) = pnt2splat[i_vtx].property();
        let ix = r0[0] as usize;
        let iy = r0[1] as usize;
        let ipix = iy * img_shape.0 + ix;
        img_data[ipix][0] = rgb[0];
        img_data[ipix][1] = rgb[1];
        img_data[ipix][2] = rgb[2];
    }
    use ::slice_of_array::SliceFlatExt; // for flat
    crate::write_png_from_float_image_rgb(path, &img_shape, (&img_data).flat())?;
    Ok(())
}
