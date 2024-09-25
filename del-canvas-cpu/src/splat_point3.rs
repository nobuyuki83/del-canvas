pub trait Splat3 {
    fn xyz(&self) -> [f32; 3];
    fn rgb(&self) -> [f32; 3];
}
pub fn draw_pix<S: Splat3, Path>(
    pnt2splat3: &[S],
    img_shape: (usize, usize),
    transform_world2ndc: &[f32; 16],
    path: Path,
) -> anyhow::Result<()>
where
    Path: AsRef<std::path::Path>,
{
    let mut img_data = vec![[0f32, 0f32, 0f32]; img_shape.0 * img_shape.1]; // black
    let transform_ndc2pix = del_geo_core::mat2x3_col_major::transform_ndc2pix(img_shape);
    for i_pnt in 0..pnt2splat3.len() {
        let xyz = pnt2splat3[i_pnt].xyz();
        let rgb = pnt2splat3[i_pnt].rgb();
        let q0 = del_geo_core::mat4_col_major::transform_homogeneous(&transform_world2ndc, &xyz)
            .unwrap();
        let r0 =
            del_geo_core::mat2x3_col_major::mult_vec3(&transform_ndc2pix, &[q0[0], q0[1], 1f32]);
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
