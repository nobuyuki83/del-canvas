pub trait Splat2 {
    fn ndc_z(&self) -> f32;
    fn property(&self) -> (&[f32; 2], f32, &[f32; 3]);
}

pub fn draw_sort_z<S: Splat2, Path>(
    pnt2splat: &[S],
    img_shape: (usize, usize),
    path: Path,
) -> anyhow::Result<()>
where
    Path: AsRef<std::path::Path>,
{
    // draw circles
    let idx2vtx = {
        let mut idx2vtx: Vec<usize> = (0..pnt2splat.len()).collect();
        idx2vtx.sort_by(|&idx0, &idx1| {
            let z0 = pnt2splat[idx0].ndc_z() + 1f32;
            let z1 = pnt2splat[idx1].ndc_z() + 1f32;
            z0.partial_cmp(&z1).unwrap()
        });
        idx2vtx
    };
    let mut img_data = vec![[0f32, 0f32, 0f32]; img_shape.0 * img_shape.1];
    for idx in 0..pnt2splat.len() {
        let i_vtx = idx2vtx[idx];
        let ndc_z = pnt2splat[i_vtx].ndc_z();
        if ndc_z <= -1f32 || ndc_z >= 1f32 {
            continue;
        }
        let (r0, rad_pix, rgb) = pnt2splat[i_vtx].property();
        let pixs = crate::rasterize_circle::pixels_in_point(
            r0[0],
            r0[1],
            rad_pix,
            img_shape.0,
            img_shape.1,
        );
        for i_pix in pixs {
            img_data[i_pix][0] = rgb[0];
            img_data[i_pix][1] = rgb[1];
            img_data[i_pix][2] = rgb[2];
        }
    }
    use ::slice_of_array::SliceFlatExt; // for flat
    crate::write_png_from_float_image_rgb(path, &img_shape, (&img_data).flat())?;
    Ok(())
}
