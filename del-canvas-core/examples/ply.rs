use del_msh_core::vtx2xyz::HasXyz;
use num_traits::cast::AsPrimitive;

fn main() -> anyhow::Result<()> {
    let vtx2xyzrgb = del_msh_core::io_ply::read_xyzrgb("/Users/nobuyuki/project/juice_box1.ply")?;
    let aabb3 = del_msh_core::vtx2xyz::aabb3_from_points(&vtx2xyzrgb);
    let aabb3: [f32; 6] = aabb3.map(|v| v.as_());
    dbg!(aabb3);

    let img_shape = (300usize, 300usize);
    let mut img_data = vec![[0f32, 0f32, 0f32]; img_shape.0 * img_shape.1];

    let cam_proj = del_geo_core::mat4_col_major::camera_perspective_blender(
        img_shape.0 as f32 / img_shape.1 as f32,
        50f32,
        0.1,
        1.0,
        true,
    );
    let cam_modelview = del_geo_core::mat4_col_major::camera_external_blender(
        &[
            (aabb3[0] + aabb3[3]) * 0.5f32,
            (aabb3[1] + aabb3[4]) * 0.5f32,
            (aabb3[2] + aabb3[5]) * 0.5f32 + 1.4f32,
        ],
        0f32,
        0f32,
        0f32,
    );
    let transform_world2ndc = del_geo_core::mat4_col_major::multmat(&cam_proj, &cam_modelview);

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
    use del_geo_core::mat4_col_major::transform_homogeneous;
    use del_geo_core::mat2x3_col_major::mult_vec3;
    for i_edge in 0..edge2vtx.len() {
        let i0_vtx = edge2vtx[i_edge][0];
        let i1_vtx = edge2vtx[i_edge][1];
        let p0 = del_geo_core::aabb3::xyz_from_hex_index(&aabb3, i0_vtx);
        let p1 = del_geo_core::aabb3::xyz_from_hex_index(&aabb3, i1_vtx);
        let q0 = transform_homogeneous(&transform_world2ndc, &p0).unwrap();
        let q1 = transform_homogeneous(&transform_world2ndc, &p1).unwrap();
        let r0 = mult_vec3(&transform_ndc2pix, &[q0[0], q0[1], 1f32]);
        let r1 = mult_vec3(&transform_ndc2pix, &[q1[0], q1[1], 1f32]);
        del_canvas_core::rasterize_line::draw_dda::<f32, [f32; 3]>(
            &mut img_data,
            img_shape.0,
            &r0,
            &r1,
            [1.0, 1.0, 1.0],
        );
    }
    for i_elem in 0..vtx2xyzrgb.len() {
        let p0 = vtx2xyzrgb[i_elem].xyz();
        let p0: [f32;3] = p0.map(|v| v.as_());
        let q0 = transform_homogeneous(&transform_world2ndc, &p0).unwrap();
        let r0 = mult_vec3(&transform_ndc2pix, &[q0[0], q0[1], 1f32]);
        let ix = r0[0] as usize;
        let iy = r0[1] as usize;
        let ipix = iy*img_shape.0 + ix;
        img_data[ipix][0] = (vtx2xyzrgb[i_elem].rgb[0] as f32) / 255.0;
        img_data[ipix][1] = (vtx2xyzrgb[i_elem].rgb[1] as f32) / 255.0;
        img_data[ipix][2] = (vtx2xyzrgb[i_elem].rgb[2] as f32) / 255.0;
    }
    use ::slice_of_array::SliceFlatExt; // for flat
    del_canvas_core::write_png_from_float_image_rgb(
        "target/ply.png",
        &img_shape,
        (&img_data).flat(),
    )?;
    Ok(())
}
