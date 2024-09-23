use rand::Rng;

#[derive(Default, Debug)]
struct Point {
    // 3d information
    pub pos_world: [f32; 3],
    pub normal_world: [f32; 3],
    pub color: [f32; 3],
    pub sigma: nalgebra::Matrix3<f32>,

    // 2d information
    pub pos_ndc: [f32; 3],
    // pub quaternion: [f32;4]
    pub w: nalgebra::Matrix2<f32>,
    // screen position
    pub s: nalgebra::Vector2<f32>,
    // aabb
    pub aabb: [f32; 4],
}

impl del_canvas_cpu::rasterize_points3::Point2dWithColor for Point {
    fn pix_coord(&self) -> [f32; 2] {
        [self.s[0], self.s[1]]
    }

    fn color(&self) -> [f32; 3] {
        self.color
    }
}

fn main() -> anyhow::Result<()> {
    let (tri2vtx, vtx2xyz, _vtx2uv) = {
        let mut obj = del_msh_core::io_obj::WavefrontObj::<usize, f32>::new();
        obj.load("asset/spot_triangulated.obj")?;
        obj.unified_xyz_uv_as_trimesh()
    };

    // define 3D points
    let mut points = {
        let mut points: Vec<Point> = vec![];
        let cumsumarea = del_msh_core::sampling::cumulative_area_sum(&tri2vtx, &vtx2xyz, 3);
        // let mut reng = rand::thread_rng();
        use rand::SeedableRng;
        let mut reng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
        for _i in 0..10000 {
            let val01_a = reng.gen::<f32>();
            let val01_b = reng.gen::<f32>();
            let barycrd =
                del_msh_core::sampling::sample_uniformly_trimesh(&cumsumarea, val01_a, val01_b);
            let tri = del_msh_core::trimesh3::to_tri3(barycrd.0, &tri2vtx, &vtx2xyz);
            let pos_world = tri.position_from_barycentric_coordinates(barycrd.1, barycrd.2);
            let normal_world = del_geo_core::vec3::normalized(&tri.normal());
            let color = [
                normal_world[0] * 0.5 + 0.5,
                normal_world[1] * 0.5 + 0.5,
                normal_world[2] * 0.5 + 0.5,
            ];
            let rad = 0.03;
            let mut sigma = nalgebra::Matrix3::<f32>::from_diagonal_element(rad * rad);
            let n = nalgebra::Vector3::<f32>::from_column_slice(&normal_world);
            let n = n.normalize();
            sigma -= n * n.transpose() * rad * rad * 0.99;
            let pnt = Point {
                pos_world,
                normal_world,
                color,
                sigma,
                ..Default::default()
            };
            points.push(pnt);
        }
        points
    };

    const TILE_SIZE: usize = 16;
    let img_shape = (TILE_SIZE * 28, TILE_SIZE * 28);
    let cam_projection = del_geo_core::mat4_col_major::camera_perspective_blender(
        img_shape.0 as f32 / img_shape.1 as f32,
        24f32,
        0.5,
        3.0,
        true,
    );
    let cam_modelview =
        del_geo_core::mat4_col_major::camera_external_blender(&[0., 0., 2.], 0., 0., 0.);
    let transform_world2ndc =
        del_geo_core::mat4_col_major::mult_mat(&cam_projection, &cam_modelview);

    // transform points
    for point in points.iter_mut() {
        point.pos_ndc = del_geo_core::mat4_col_major::transform_homogeneous(
            &transform_world2ndc,
            &point.pos_world,
        )
        .unwrap();
    }
    // sort points depth
    points.sort_by(|a, b| a.pos_ndc[2].partial_cmp(&b.pos_ndc[2]).unwrap());

    // projects points & covariance 2D
    for point in points.iter_mut() {
        // screen position
        {
            let x = (point.pos_ndc[0] + 1.0) * 0.5 * (img_shape.0 as f32);
            let y = (1.0 - point.pos_ndc[1]) * 0.5 * (img_shape.1 as f32);
            point.s = nalgebra::Vector2::new(x, y);
        }
        let w = nalgebra::Matrix3::<f32>::new(
            cam_modelview[0],
            cam_modelview[4],
            cam_modelview[8],
            cam_modelview[1],
            cam_modelview[5],
            cam_modelview[9],
            cam_modelview[2],
            cam_modelview[6],
            cam_modelview[10],
        );
        use del_geo_core::mat4_col_major::transform_homogeneous;
        let q = transform_homogeneous(&cam_modelview, &point.pos_world).unwrap();
        let cam_projection = nalgebra::Matrix4::<f32>::from_column_slice(&cam_projection);
        let q = nalgebra::Vector3::<f32>::from_column_slice(&q);
        let j = del_geo_nalgebra::mat4::jacobian_transform(&cam_projection, &q);
        let r = nalgebra::Matrix2x3::<f32>::new(
            0.5 * (img_shape.0 as f32),
            0.,
            0.,
            0.,
            -0.5 * (img_shape.1 as f32),
            0.,
        );
        let w0 = r * j * w * point.sigma * w.transpose() * j.transpose() * r.transpose();
        let w0 = w0.try_inverse().unwrap();
        point.w = w0;
        // screen aabb
        {
            let aabb = del_geo_core::mat2_sym::aabb2(&[w0.m11, w0.m12, w0.m22]);
            let aabb = del_geo_core::aabb2::scale(&aabb, 3.0);
            let aabb = del_geo_core::aabb2::translate(&aabb, &[point.s[0], point.s[1]]);
            point.aabb = aabb;
        }
    }

    // visualize as points
    {
        /*
        let mut img_data = vec![0f32; img_size.1 * img_size.0 * 3];
        for point in points.iter() {
            let x = (point.pos_ndc[0] + 1.0) * 0.5 * (img_size.0 as f32);
            let y = (1.0 - point.pos_ndc[1]) * 0.5 * (img_size.1 as f32);
            let i_x = x as usize;
            let i_y = y as usize;
            img_data[(i_y * img_size.0 + i_x) * 3 + 0] = point.color[0];
            img_data[(i_y * img_size.0 + i_x) * 3 + 1] = point.color[1];
            img_data[(i_y * img_size.0 + i_x) * 3 + 2] = point.color[2];
        }
         */
        let img_data = del_canvas_cpu::rasterize_points3::points(&img_shape, &points);
        del_canvas_cpu::write_png_from_float_image_rgb(
            "target/points3d_pix.png",
            &img_shape,
            &img_data,
        )?;
    }

    {
        // splatting Gaussian with tile-based acceleration
        println!("gaussian_tile");
        let now = std::time::Instant::now();
        let tile_shape: (usize, usize) = (img_shape.0 / TILE_SIZE, img_shape.1 / TILE_SIZE);
        let mut tile2gauss: Vec<Vec<usize>> = vec![vec!(); tile_shape.0 * tile_shape.1];
        for (i_gauss, point) in points.iter().enumerate().rev() {
            let tiles = del_geo_core::aabb2::overlapping_tiles(&point.aabb, TILE_SIZE, tile_shape);
            for i_tile in tiles {
                tile2gauss[i_tile].push(i_gauss);
            }
        }
        let mut img_data = vec![0f32; img_shape.1 * img_shape.0 * 3];
        for ih in 0..img_shape.1 {
            for iw in 0..img_shape.0 {
                let t = nalgebra::Vector2::<f32>::new(iw as f32 + 0.5, ih as f32 + 0.5);
                let mut alpha_sum = 0f32;
                let mut alpha_occu = 1f32;
                let i_tile = (ih / TILE_SIZE) * tile_shape.0 + (iw / TILE_SIZE);
                for &i_point in tile2gauss[i_tile].iter() {
                    let point = &points[i_point];
                    // front to back
                    if !del_geo_core::aabb2::is_inlcude_point(&point.aabb, &[t[0], t[1]]) {
                        continue;
                    }
                    let t0 = t - point.s;
                    let e = (t0.transpose() * point.w * t0).x;
                    let e = (-0.5 * e).exp();
                    let e_out = alpha_occu * e;
                    img_data[(ih * img_shape.0 + iw) * 3 + 0] += point.color[0] * e_out;
                    img_data[(ih * img_shape.0 + iw) * 3 + 1] += point.color[1] * e_out;
                    img_data[(ih * img_shape.0 + iw) * 3 + 2] += point.color[2] * e_out;
                    alpha_occu *= 1f32 - e;
                    alpha_sum += e_out;
                    if alpha_sum > 0.999 {
                        break;
                    }
                }
            }
        }

        /*
               let pix2rgb = |i_pix: usize| -> [f32;3] {
                   let ih = i_pix / img_size.0;
                   let iw = i_pix - ih * img_size.0;
                   let t = nalgebra::Vector2::<f32>::new(iw as f32 + 0.5, ih as f32 + 0.5);
                   let mut alpha_sum = 0f32;
                   let mut alpha_occu = 1f32;
                   let i_tile = (ih / 16) * tile_size.1 + (iw / 16);
                   let mut rgb = [0f32;3];
                   for &i_point in tile2gauss[i_tile].iter() {
                       let point = &points[i_point];
                       // front to back
                       if !del_geo_core::aabb2::is_inlcude_point(&point.aabb, &[t[0], t[1]]) { continue; }
                       let t0 = t - point.s;
                       let e = (t0.transpose() * point.w * t0).x;
                       let e = (-0.5 * e).exp();
                       let e_out = alpha_occu * e;
                       rgb[0] += point.color[0] * e_out;
                       rgb[1] += point.color[1] * e_out;
                       rgb[2] += point.color[2] * e_out;
                       alpha_occu *= 1f32 - e;
                       alpha_sum += e_out;
                       if alpha_sum > 0.999 { break; }
                   }
                   rgb
               };
               use rayon::iter::ParallelIterator;
               let img_data: Vec<f32> = (0..img_size.0 * img_size.1)
                   .into_par_iter()
                   .flat_map(pix2rgb)
                   .collect();
        */
        del_canvas_cpu::write_png_from_float_image_rgb(
            "target/points3d_gaussian_tile.png",
            &img_shape,
            &img_data,
        )?;
        println!("   Elapsed gaussian_tile: {:.2?}", now.elapsed());
    }

    {
        // visualize as Gaussian without tile acceleration
        println!("gaussian_naive");
        let now = std::time::Instant::now();
        let mut img_data = vec![0f32; img_shape.1 * img_shape.0 * 3];
        for ih in 0..img_shape.1 {
            for iw in 0..img_shape.0 {
                let t = nalgebra::Vector2::<f32>::new(iw as f32 + 0.5, ih as f32 + 0.5);
                let mut alpha_sum = 0f32;
                let mut alpha_occu = 1f32;
                for point in points.iter().rev() {
                    // front to back
                    if !del_geo_core::aabb2::is_inlcude_point(&point.aabb, &[t[0], t[1]]) {
                        continue;
                    }
                    let t0 = t - point.s;
                    let e = (t0.transpose() * point.w * t0).x;
                    let e = (-0.5 * e).exp();
                    let e_out = alpha_occu * e;
                    img_data[(ih * img_shape.0 + iw) * 3 + 0] += point.color[0] * e_out;
                    img_data[(ih * img_shape.0 + iw) * 3 + 1] += point.color[1] * e_out;
                    img_data[(ih * img_shape.0 + iw) * 3 + 2] += point.color[2] * e_out;
                    alpha_occu *= 1f32 - e;
                    alpha_sum += e_out;
                    if alpha_sum > 0.999 {
                        break;
                    }
                }
            }
        }
        del_canvas_cpu::write_png_from_float_image_rgb(
            "target/points3d_gaussian.png",
            &img_shape,
            &img_data,
        )?;
        println!("   Elapsed gaussian_naive: {:.2?}", now.elapsed());
    }

    {
        // visualize Gaussian as ellipsoid
        println!("gaussian_ellipse");
        let now = std::time::Instant::now();
        let mut img_data = vec![0f32; img_shape.1 * img_shape.0 * 3];
        for ih in 0..img_shape.1 {
            for iw in 0..img_shape.0 {
                let t = nalgebra::Vector2::<f32>::new(iw as f32 + 0.5, ih as f32 + 0.5);
                for point in points.iter().rev() {
                    if !del_geo_core::aabb2::is_inlcude_point(&point.aabb, &[t[0], t[1]]) {
                        continue;
                    }
                    let t0 = t - point.s;
                    let a = (t0.transpose() * point.w * t0).x;
                    if a > 1f32 {
                        continue;
                    }
                    img_data[(ih * img_shape.0 + iw) * 3 + 0] = point.color[0];
                    img_data[(ih * img_shape.0 + iw) * 3 + 1] = point.color[1];
                    img_data[(ih * img_shape.0 + iw) * 3 + 2] = point.color[2];
                    break;
                }
            }
        }
        del_canvas_cpu::write_png_from_float_image_rgb(
            "target/points3d_ellipse.png",
            &img_shape,
            &img_data,
        )?;
        println!("   Elapsed ellipse: {:.2?}", now.elapsed());
    }

    Ok(())
}
