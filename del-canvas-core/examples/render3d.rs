fn main() -> anyhow::Result<()>{
    let (tri2vtx, vtx2xyz, _vtx2uv) = {
        let mut obj = del_msh_core::io_obj::WavefrontObj::<usize, f32>::new();
        obj.load("examples/asset/spot_triangulated.obj")?;
        obj.unified_xyz_uv_as_trimesh()
    };
    let img_size = {
        const TILE_SIZE: usize = 16;
        (TILE_SIZE * 28, TILE_SIZE * 28)
    };
    let cam_projection = del_geo_core::mat4_col_major::camera_perspective_blender(
        img_size.0 as f32 / img_size.1 as f32,
        24f32,
        0.5,
        3.0,
    );
    let cam_modelview =
        del_geo_core::mat4_col_major::camera_external_blender(&[0., 0., 2.], 0., 0., 0.);
    let transform_world2ndc =
        del_geo_core::mat4_col_major::multmat(&cam_projection, &cam_modelview);
    let transform_ndc2world =
        del_geo_core::mat4_col_major::try_inverse(&transform_world2ndc).unwrap();
    /*
    {
        let vtx2xyz2 = del_msh_core::vtx2xyz::transform(&vtx2xyz, &transform_world2ndc);
        del_msh_core::io_obj::save_tri2vtx_vtx2xyz("target/hoge.obj", &tri2vtx, &vtx2xyz2, 3);
    }
     */
    let bvhnodes = del_msh_core::bvhnodes_morton::from_triangle_mesh(&tri2vtx, &vtx2xyz, 3);
    let aabbs = del_msh_core::aabbs3::from_uniform_mesh_with_bvh(
        0,
        &bvhnodes,
        Some((&tri2vtx, 3)),
        &vtx2xyz,
        None,
    );
    let pix2tri = del_canvas_core::raycast_trimesh3::pix2tri(
        &tri2vtx,
        &vtx2xyz,
        &bvhnodes,
        &aabbs,
        &img_size,
        &transform_ndc2world,
    );

    {
        // render normalmap
        let pix2rgb = del_canvas_core::raycast_trimesh3::render_normalmap_pix2tri(
            img_size,
            &cam_modelview,
            &tri2vtx,
            &vtx2xyz,
            &pix2tri,
        );
        del_canvas_core::write_png_from_float_image_rgb(
            "target/render3d_normalmap.png",
            &img_size,
            &pix2rgb,
        );
    }

    // render depth
    {
        let mut img_data = vec![0f32; img_size.0 * img_size.1];
        del_canvas_core::raycast_trimesh3::render_depth_bvh(
            img_size,
            &mut img_data,
            &transform_ndc2world,
            &tri2vtx,
            &vtx2xyz,
            &bvhnodes,
            &aabbs,
        );
        del_canvas_core::write_png_from_float_image_grayscale(
            "target/render3d_depth.png",
            &img_size,
            &img_data,
        );
    }

    /*
    {
        // render texture
        let (tex_data, tex_shape, depth)
            = del_canvas::load_image_as_float_array("examples/asset/spot_texture.png");
        dbg!(tex_shape);
        let mut img_data = vec![0f32; img_size.0 * img_size.1];
        del_canvas::raycast_trimesh3::render_depth_bvh(
            img_size,
            &mut img_data,
            &transform_ndc2world,
            &tri2vtx,
            &vtx2xyz,
            &bvhnodes,
            &aabbs,
        );
    }
     */

    Ok(())
}
