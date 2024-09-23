use num_traits::cast::AsPrimitive;

#[derive(Clone, Default)]
pub struct XyzRgb {
    xyz: [f32; 3],
    rgb: [u8; 3],
}

impl del_msh_core::io_ply::XyzRgb for XyzRgb {
    fn new(xyz: [f64; 3], rgb: [u8; 3]) -> Self {
        XyzRgb {
            xyz: xyz.map(|v| v as f32),
            rgb,
        }
    }
}

impl del_msh_core::vtx2point::HasXyz<f32> for XyzRgb {
    fn xyz(&self) -> &[f32; 3] {
        &self.xyz
    }
}

#[derive(Clone)]
struct Circle {
    ndc_z: f32,
    pos_pix: [f32; 2],
    rad_pix: f32,
    rgb: [f32; 3],
}

fn draw_circles(
    vtx2splat: &[Circle],
    img_shape: (usize, usize),
    transform_world2ndc: &[f32; 16],
    aabb3: &[f32; 6],
) -> anyhow::Result<()> {
    // draw pixels
    let idx2vtx = {
        let mut idx2vtx: Vec<usize> = (0..vtx2splat.len()).collect();
        idx2vtx.sort_by(|&idx0, &idx1| {
            vtx2splat[idx0]
                .ndc_z
                .partial_cmp(&vtx2splat[idx1].ndc_z)
                .unwrap()
        });
        idx2vtx
    };
    let mut img_data = vec![[0f32, 0f32, 0f32]; img_shape.0 * img_shape.1];
    del_canvas_cpu::rasterize_aabb3::wireframe_dda(
        &mut img_data,
        img_shape,
        &transform_world2ndc,
        &aabb3,
        [1.0, 1.0, 1.0],
    );
    for i_idx in 0..vtx2splat.len() {
        let i_vtx = idx2vtx[i_idx];
        let r0 = vtx2splat[i_vtx].pos_pix;
        let ix = r0[0] as usize;
        let iy = r0[1] as usize;
        let ipix = iy * img_shape.0 + ix;
        img_data[ipix][0] = vtx2splat[i_vtx].rgb[0];
        img_data[ipix][1] = vtx2splat[i_vtx].rgb[1];
        img_data[ipix][2] = vtx2splat[i_vtx].rgb[2];
    }
    use ::slice_of_array::SliceFlatExt; // for flat
    del_canvas_cpu::write_png_from_float_image_rgb(
        "target/ply_pixel.png",
        &img_shape,
        (&img_data).flat(),
    )?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    // let path = "/Users/nobuyuki/project/juice_box1.ply";
    let file_path = "asset/juice_box.ply";
    let pnt2xyzrgb = del_msh_core::io_ply::read_xyzrgb::<_, XyzRgb>(file_path)?;
    let aabb3 = del_msh_core::vtx2point::aabb3_from_points(&pnt2xyzrgb);
    let aabb3: [f32; 6] = aabb3.map(|v| v.as_());
    let img_shape = (2000usize, 1200usize);
    let transform_world2ndc = {
        let cam_proj = del_geo_core::mat4_col_major::camera_perspective_blender(
            img_shape.0 as f32 / img_shape.1 as f32,
            50f32,
            0.1,
            2.0,
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
        del_geo_core::mat4_col_major::mult_mat(&cam_proj, &cam_modelview)
    };
    let transform_ndc2pix = del_geo_core::mat2x3_col_major::transform_ndc2pix(img_shape);
    let radius = 0.0015;
    use del_geo_core::mat2x3_col_major::mult_vec3;
    use del_geo_core::mat4_col_major::transform_homogeneous;
    let pnt2circle = {
        use del_msh_core::vtx2point::HasXyz;
        let mut vtx2splat = vec![
            Circle {
                ndc_z: 0f32,
                pos_pix: [0f32; 2],
                rad_pix: 0f32,
                rgb: [0f32; 3],
            };
            pnt2xyzrgb.len()
        ];
        for i_elem in 0..pnt2xyzrgb.len() {
            let p0 = pnt2xyzrgb[i_elem].xyz();
            let p0: [f32; 3] = p0.map(|v| v.as_());
            let q0 = transform_homogeneous(&transform_world2ndc, &p0).unwrap();
            let r0 = mult_vec3(&transform_ndc2pix, &[q0[0], q0[1], 1f32]);
            let rad_pix = {
                let dqdp =
                    del_geo_core::mat4_col_major::jacobian_transform(&transform_world2ndc, &p0);
                let dqdp = del_geo_core::mat3_col_major::try_inverse(&dqdp).unwrap();
                let dx = [dqdp[0], dqdp[1], dqdp[2]];
                let dy = [dqdp[3], dqdp[4], dqdp[5]];
                let rad_pix_x =
                    (1.0 / del_geo_core::vec3::norm(&dx)) * 0.5 * img_shape.0 as f32 * radius;
                let rad_pxi_y =
                    (1.0 / del_geo_core::vec3::norm(&dy)) * 0.5 * img_shape.1 as f32 * radius;
                0.5 * (rad_pix_x + rad_pxi_y)
            };
            // dbg!(rad_pix);
            vtx2splat[i_elem].ndc_z = q0[2];
            vtx2splat[i_elem].pos_pix = r0;
            vtx2splat[i_elem].rad_pix = rad_pix;
            vtx2splat[i_elem].rgb = [
                (pnt2xyzrgb[i_elem].rgb[0] as f32) / 255.0,
                (pnt2xyzrgb[i_elem].rgb[1] as f32) / 255.0,
                (pnt2xyzrgb[i_elem].rgb[2] as f32) / 255.0,
            ];
        }
        vtx2splat
    };
    draw_circles(&pnt2circle, img_shape, &transform_world2ndc, &aabb3)?;
    {
        let now = std::time::Instant::now();
        // draw circles
        let idx2vtx = {
            let mut idx2vtx: Vec<usize> = (0..pnt2circle.len()).collect();
            idx2vtx.sort_by(|&idx0, &idx1| {
                pnt2circle[idx0]
                    .ndc_z
                    .partial_cmp(&pnt2circle[idx1].ndc_z)
                    .unwrap()
            });
            idx2vtx
        };
        let mut img_data = vec![[0f32, 0f32, 0f32]; img_shape.0 * img_shape.1];
        del_canvas_cpu::rasterize_aabb3::wireframe_dda(
            &mut img_data,
            img_shape,
            &transform_world2ndc,
            &aabb3,
            [1.0, 1.0, 1.0],
        );
        use del_msh_core::vtx2point::HasXyz;
        for idx in 0..pnt2xyzrgb.len() {
            let i_vtx = idx2vtx[idx];
            let p0 = pnt2xyzrgb[i_vtx].xyz();
            let p0: [f32; 3] = p0.map(|v| v.as_());
            let q0 = del_geo_core::mat4_col_major::transform_homogeneous(&transform_world2ndc, &p0)
                .unwrap();
            let r0 = del_geo_core::mat2x3_col_major::mult_vec3(
                &transform_ndc2pix,
                &[q0[0], q0[1], 1f32],
            );
            let pixs = del_canvas_cpu::rasterize_circle::pixels_in_point(
                r0[0],
                r0[1],
                pnt2circle[i_vtx].rad_pix,
                img_shape.0,
                img_shape.1,
            );
            for i_pix in pixs {
                img_data[i_pix][0] = (pnt2xyzrgb[i_vtx].rgb[0] as f32) / 255.0;
                img_data[i_pix][1] = (pnt2xyzrgb[i_vtx].rgb[1] as f32) / 255.0;
                img_data[i_pix][2] = (pnt2xyzrgb[i_vtx].rgb[2] as f32) / 255.0;
            }
        }
        use ::slice_of_array::SliceFlatExt; // for flat
        del_canvas_cpu::write_png_from_float_image_rgb(
            "target/ply_points_naive.png",
            &img_shape,
            (&img_data).flat(),
        )?;
        println!("   Elapsed gaussian_tile: {:.2?}", now.elapsed());
    }
    {
        // draw circles with tiles
        let now = std::time::Instant::now();
        const TILE_SIZE: usize = 16;
        assert_eq!(img_shape.0 % TILE_SIZE, 0);
        assert_eq!(img_shape.0 % TILE_SIZE, 0);
        let tile_shape = (img_shape.0 / TILE_SIZE, img_shape.1 / TILE_SIZE);
        let num_tile = tile_shape.0 * tile_shape.1;
        let mut tile2ind = vec![0usize; num_tile + 1];
        let mut num_ind = 0;
        for i_vtx in 0..pnt2circle.len() {
            let p0 = pnt2circle[i_vtx].pos_pix;
            let rad = pnt2circle[i_vtx].rad_pix;
            let aabb2 = del_geo_core::aabb2::from_point(&p0, rad);
            let tiles = del_geo_core::aabb2::overlapping_tiles(&aabb2, TILE_SIZE, tile_shape);
            num_ind += tiles.len();
            for &i_tile in tiles.iter() {
                tile2ind[i_tile + 1] += 1;
            }
        }
        for i_tile in 0..num_tile {
            tile2ind[i_tile + 1] += tile2ind[i_tile];
        }
        let num_ind = tile2ind[num_tile];
        dbg!(pnt2circle.len(), num_ind);
        let mut ind2vtx = vec![0usize; num_ind];
        let mut ind2tiledepth = Vec::<(usize, usize, f32)>::with_capacity(num_ind);
        for i_vtx in 0..pnt2circle.len() {
            let p0 = pnt2circle[i_vtx].pos_pix;
            let rad = pnt2circle[i_vtx].rad_pix;
            let depth = pnt2circle[i_vtx].ndc_z;
            let aabb2 = del_geo_core::aabb2::from_point(&p0, rad);
            let tiles = del_geo_core::aabb2::overlapping_tiles(&aabb2, TILE_SIZE, tile_shape);
            for &i_tile in tiles.iter() {
                ind2vtx[ind2tiledepth.len()] = i_vtx;
                ind2tiledepth.push((i_vtx, i_tile, depth));
            }
        }
        assert_eq!(ind2tiledepth.len(), num_ind);
        ind2tiledepth.sort_by(|&a, &b| {
            if a.1 == b.1 {
                a.2.partial_cmp(&b.2).unwrap()
            } else {
                a.1.cmp(&b.1)
            }
        });
        for iind in 0..ind2tiledepth.len() {
            ind2vtx[iind] = ind2tiledepth[iind].0;
        }
        //
        let mut img_data = vec![[0f32, 0f32, 0f32]; img_shape.0 * img_shape.1];
        del_canvas_cpu::rasterize_aabb3::wireframe_dda(
            &mut img_data,
            img_shape,
            &transform_world2ndc,
            &aabb3,
            [1.0, 1.0, 1.0],
        );
        for (iw, ih) in itertools::iproduct!(0..img_shape.0, 0..img_shape.1) {
            let i_tile = (ih / TILE_SIZE) * tile_shape.0 + (iw / TILE_SIZE);
            let i_pix = ih * img_shape.0 + iw;
            for &i_vtx in &ind2vtx[tile2ind[i_tile]..tile2ind[i_tile + 1]] {
                let p0 = pnt2circle[i_vtx].pos_pix;
                let rad = pnt2circle[i_vtx].rad_pix;
                let p1 = [iw as f32 + 0.5f32, ih as f32 + 0.5f32];
                if del_geo_core::edge2::length(&p0, &p1) > rad {
                    continue;
                }
                img_data[i_pix][0] = (pnt2xyzrgb[i_vtx].rgb[0] as f32) / 255.0;
                img_data[i_pix][1] = (pnt2xyzrgb[i_vtx].rgb[1] as f32) / 255.0;
                img_data[i_pix][2] = (pnt2xyzrgb[i_vtx].rgb[2] as f32) / 255.0;
            }
        }
        use ::slice_of_array::SliceFlatExt; // for flat
        del_canvas_cpu::write_png_from_float_image_rgb(
            "target/ply_points_tile.png",
            &img_shape,
            (&img_data).flat(),
        )?;
        println!("   Elapsed gaussian_tile: {:.2?}", now.elapsed());
    }
    Ok(())
}
