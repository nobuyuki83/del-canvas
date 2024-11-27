fn main() -> anyhow::Result<()> {
    let (tex_shape, tex_data) = {
        let pfm = del_raycast::io_pfm::PFM::read_from(
            "asset/material-testball/textures/envmap.pfm",
        )?;
        ((pfm.w, pfm.h), pfm.data)
    };
    {
        use image::Pixel;
        let img: Vec<image::Rgb<f32>> = tex_data
            .chunks(3)
            .map(|rgb| *image::Rgb::<f32>::from_slice(rgb))
            .collect();
        use image::codecs::hdr::HdrEncoder;
        let file = std::fs::File::create("target/04_env_light_pfm.hdr").unwrap();
        let enc = HdrEncoder::new(file);
        let _ = enc.encode(&img, tex_shape.0, tex_shape.1);
    }
    // --------------------
    let camera_fov = 20.0;
    // let transform_cam_lcl2glbl = del_geo_core::mat4_col_major::from_translate(&[0., 0., -5.]);
    let transform_cam_glbl2lcl: [f32; 16] = [
        0.721367, -0.373123, -0.583445, -0., -0., 0.842456, -0.538765, -0., -0.692553, -0.388647,
        -0.60772, -0., 0.0258668, -0.29189, 5.43024, 1.,
    ];
    let transform_cam_lcl2glbl =
        del_geo_core::mat4_col_major::try_inverse(&transform_cam_glbl2lcl).unwrap();
    let transform_env = [
        -0.386527, 0., 0.922278, 0., -0.922278, 0., -0.386527, 0., 0., 1., 0., 0., 0., 0., 0., 1.,
    ];
    let transform_env: [f32; 16] = {
        let m = nalgebra::Matrix4::<f32>::from_column_slice(&transform_env);
        let m = m.try_inverse().unwrap();
        // let transform_env = del_geo_core::mat4_col_major::try_inverse(&transform_env).unwrap();
        m.as_slice().try_into().unwrap()
    };
    // dbg!(transform_cam_lcl2glbl);
    let img_shape = (640, 360);
    let mut img = Vec::<image::Rgb<f32>>::new();
    img.resize(img_shape.0 * img_shape.1, image::Rgb([0_f32; 3]));
    for ih in 0..img_shape.1 {
        for iw in 0..img_shape.0 {
            let (ray_org, ray_dir) = del_raycast::cam_pbrt::cast_ray(
                iw,
                ih,
                img_shape,
                camera_fov,
                transform_cam_lcl2glbl,
            );
            let sphere_cntr = [0.15, 0.50, 0.16];
            let t = del_geo_nalgebra::sphere::intersection_ray(
                &nalgebra::Vector3::<f32>::from(sphere_cntr),
                0.7,
                &nalgebra::Vector3::<f32>::from(ray_org),
                &nalgebra::Vector3::<f32>::from(ray_dir),
            );
            if let Some(t) = t {
                use del_geo_core::vec3;
                let pos = vec3::axpy::<f32>(t, &ray_dir, &ray_org);
                let nrm = vec3::sub(&pos, &sphere_cntr);
                let hit_nrm = vec3::normalized(&nrm);
                let refl = vec3::mirror_reflection(&ray_dir, &hit_nrm);
                let refl = vec3::normalized(&refl);
                let env =
                    del_geo_core::mat4_col_major::transform_homogeneous(&transform_env, &refl)
                        .unwrap();
                // let tex_coord = unit2_from_uvec3_equal_area(&[-env[0], -env[1], -env[2]]);
                let tex_coord = del_geo_core::uvec3::map_to_unit2_equal_area(&env);
                let i_u = (tex_coord[0] * tex_shape.0 as f32).floor() as usize;
                let i_v = (tex_coord[1] * tex_shape.1 as f32).floor() as usize;
                let i_u = i_u.clamp(0, tex_shape.0 - 1);
                let i_v = i_v.clamp(0, tex_shape.1 - 1);
                let i_tex = i_v * tex_shape.0 + i_u;
                img[ih * img_shape.0 + iw].0[0] = tex_data[i_tex * 3 + 0];
                img[ih * img_shape.0 + iw].0[1] = tex_data[i_tex * 3 + 1];
                img[ih * img_shape.0 + iw].0[2] = tex_data[i_tex * 3 + 2];
            } else {
                let nrm = del_geo_core::vec3::normalized(&ray_dir);
                let env = del_geo_core::mat4_col_major::transform_homogeneous(&transform_env, &nrm)
                    .unwrap();
                // let tex_coord = unit2_from_uvec3_octahedra(&[-env[0], env[1], -env[2]]);
                let tex_coord = del_geo_core::uvec3::map_to_unit2_equal_area(&env);
                let i_u = (tex_coord[0] * tex_shape.0 as f32).floor() as usize;
                let i_v = (tex_coord[1] * tex_shape.1 as f32).floor() as usize;
                let i_u = i_u.clamp(0, tex_shape.0 - 1);
                let i_v = i_v.clamp(0, tex_shape.1 - 1);
                let i_tex = i_v * tex_shape.0 + i_u;
                img[ih * img_shape.0 + iw].0[0] = tex_data[i_tex * 3 + 0];
                img[ih * img_shape.0 + iw].0[1] = tex_data[i_tex * 3 + 1];
                img[ih * img_shape.0 + iw].0[2] = tex_data[i_tex * 3 + 2];
            }
        }
    }
    {
        use image::codecs::hdr::HdrEncoder;
        let file = std::fs::File::create("target/04_env_light.hdr").unwrap();
        let enc = HdrEncoder::new(file);
        let _ = enc.encode(&img, img_shape.0, img_shape.1);
    }
    Ok(())
}
