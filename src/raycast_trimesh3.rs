pub fn render_depth_bvh(
    image_size: (usize, usize),
    img_data: &mut [f32],
    transform_ndc_to_world: &[f32; 16],
    tri2vtx: &[usize],
    vtx2xyz: &[f32],
    bvhnodes: &[usize],
    aabbs: &[f32])
{
    let transform_world_to_ndc: [f32; 16]
        = nalgebra::Matrix4::<f32>::from_column_slice(transform_ndc_to_world)
        .try_inverse().unwrap()
        .as_slice().try_into().unwrap();
    let (width, height) = image_size;
    for ih in 0..height {
        for iw in 0..width {
            let (ray_org, ray_dir)
                = crate::cam3::ray3_homogeneous(
                (iw, ih), image_size, &transform_ndc_to_world);
            let mut hits: Vec<(f32, usize)> = vec!();
            del_msh::bvh3::intersection_ray(
                &mut hits,
                &tri2vtx, &vtx2xyz,
                &ray_org, &ray_dir,
                0, &bvhnodes, &aabbs);
            hits.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            let Some(&(depth, i_tri)) = hits.first() else { continue; };
            let pos = del_geo::vec3::axpy_(depth, &ray_dir, &ray_org);
            let ndc = del_geo::mat4::transform_homogeneous(&transform_world_to_ndc, &pos).unwrap();
            let depth_ndc = (ndc[2] + 1f32) * 0.5f32;
            img_data[ih * width + iw] = depth_ndc;
        }
    }
}