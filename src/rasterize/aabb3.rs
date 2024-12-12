pub fn wireframe_dda<DATA>(
    img_data: &mut [DATA],
    img_shape: (usize, usize),
    transform_world2ndc: &[f32; 16],
    aabb3: &[f32; 6],
    val: DATA,
) where
    DATA: Copy,
{
    let transform_ndc2pix = [
        0.5 * (img_shape.0 as f32),
        0.,
        0.,
        -0.5 * (img_shape.1 as f32),
        0.5 * (img_shape.0 as f32),
        0.5 * (img_shape.1 as f32),
    ];
    let edge2vtx = [
        [0, 1],
        [1, 2],
        [2, 3],
        [3, 0],
        [4, 5],
        [5, 6],
        [6, 7],
        [7, 4],
        [0, 4],
        [1, 5],
        [2, 6],
        [3, 7],
    ];
    use del_geo_core::mat2x3_col_major::mult_vec3;
    use del_geo_core::mat4_col_major::transform_homogeneous;
    for i_edge in 0..edge2vtx.len() {
        let i0_vtx = edge2vtx[i_edge][0];
        let i1_vtx = edge2vtx[i_edge][1];
        let p0 = del_geo_core::aabb3::xyz_from_hex_index(&aabb3, i0_vtx);
        let p1 = del_geo_core::aabb3::xyz_from_hex_index(&aabb3, i1_vtx);
        let q0 = transform_homogeneous(&transform_world2ndc, &p0).unwrap();
        let q1 = transform_homogeneous(&transform_world2ndc, &p1).unwrap();
        let r0 = mult_vec3(&transform_ndc2pix, &[q0[0], q0[1], 1f32]);
        let r1 = mult_vec3(&transform_ndc2pix, &[q1[0], q1[1], 1f32]);
        crate::rasterize::line::draw_dda::<f32, DATA>(img_data, img_shape.0, &r0, &r1, val);
    }
}
