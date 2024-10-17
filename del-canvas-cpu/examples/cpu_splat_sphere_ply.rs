use num_traits::cast::AsPrimitive;

#[derive(Clone, Default)]
pub struct Splat3 {
    xyz: [f32; 3],
    rgb: [u8; 3],
}

impl del_msh_core::io_ply::XyzRgb for Splat3 {
    fn new(xyz: [f64; 3], rgb: [u8; 3]) -> Self {
        Splat3 {
            xyz: xyz.map(|v| v as f32),
            rgb,
        }
    }
}

impl del_msh_core::vtx2point::HasXyz<f32> for Splat3 {
    fn xyz(&self) -> &[f32; 3] {
        &self.xyz
    }
}

#[derive(Clone)]
struct Splat2 {
    ndc_z: f32,
    pos_pix: [f32; 2],
    rad_pix: f32,
    rgb: [f32; 3],
}

impl del_canvas_cpu::splat_circle::Splat2 for Splat2 {
    fn ndc_z(&self) -> f32 {
        self.ndc_z
    }
    fn property(&self) -> (&[f32; 2], f32, &[f32; 3]) {
        (&self.pos_pix, self.rad_pix, &self.rgb)
    }
}

impl del_canvas_cpu::splat_point2::Splat2 for Splat2 {
    fn pos2_rgb(&self) -> ([f32; 2], [f32; 3]) {
        (self.pos_pix, self.rgb)
    }
}

impl del_canvas_cpu::splat_point2::NdcZ for Splat2 {
    fn ndc_z(&self) -> f32 { self.ndc_z }
}

impl del_canvas_cpu::tile_acceleration::Splat2 for Splat2 {
    fn aabb(&self) -> [f32; 4] {
        let p0 = self.pos_pix;
        let rad = self.rad_pix;
        del_geo_core::aabb2::from_point(&p0, rad)
    }
    fn ndc_z(&self) -> f32 {
        self.ndc_z
    }
}

fn main() -> anyhow::Result<()> {
    // let path = "/Users/nobuyuki/project/juice_box1.ply";
    let file_path = "asset/juice_box.ply";
    let pnt2splat3 = del_msh_core::io_ply::read_xyzrgb::<_, Splat3>(file_path)?;
    let aabb3 = del_msh_core::vtx2point::aabb3_from_points(&pnt2splat3);
    let aabb3: [f32; 6] = aabb3.map(|v| v.as_());
    let img_shape = (1600usize+1, 960usize+1);
    let transform_world2ndc = {
        let cam_proj = del_geo_core::mat4_col_major::camera_perspective_blender(
            img_shape.0 as f32 / img_shape.1 as f32,
            50f32,
            0.1,
            3.0,
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
    let pnt2splat2 = {
        // pnt2splat2 from pnt2splat3
        use del_msh_core::vtx2point::HasXyz;
        let mut vtx2splat = vec![
            Splat2 {
                ndc_z: 0f32,
                pos_pix: [0f32; 2],
                rad_pix: 0f32,
                rgb: [0f32; 3],
            };
            pnt2splat3.len()
        ];
        for i_elem in 0..pnt2splat3.len() {
            let pos_world0 = pnt2splat3[i_elem].xyz();
            let pos_world0: [f32; 3] = pos_world0.map(|v| v.as_());
            let ndc0 = del_geo_core::mat4_col_major::transform_homogeneous(&transform_world2ndc, &pos_world0).unwrap();
            let pos_pix = del_geo_core::mat2x3_col_major::mult_vec3(&transform_ndc2pix, &[ndc0[0], ndc0[1], 1f32]);
            let rad_pix = {
                let dqdp =
                    del_geo_core::mat4_col_major::jacobian_transform(&transform_world2ndc, &pos_world0);
                let dqdp = del_geo_core::mat3_col_major::try_inverse(&dqdp).unwrap();
                let dx = [dqdp[0], dqdp[1], dqdp[2]];
                let dy = [dqdp[3], dqdp[4], dqdp[5]];
                let rad_pix_x =
                    (1.0 / del_geo_core::vec3::norm(&dx)) * 0.5 * img_shape.0 as f32 * radius;
                let rad_pxi_y =
                    (1.0 / del_geo_core::vec3::norm(&dy)) * 0.5 * img_shape.1 as f32 * radius;
                0.5 * (rad_pix_x + rad_pxi_y)
            };

            vtx2splat[i_elem].ndc_z = ndc0[2];
            vtx2splat[i_elem].pos_pix = pos_pix;
            vtx2splat[i_elem].rad_pix = rad_pix;
            vtx2splat[i_elem].rgb = [
                (pnt2splat3[i_elem].rgb[0] as f32) / 255.0,
                (pnt2splat3[i_elem].rgb[1] as f32) / 255.0,
                (pnt2splat3[i_elem].rgb[2] as f32) / 255.0,
            ];
        }
        vtx2splat
    };
    del_canvas_cpu::splat_point2::draw_pix_sort_z(
        &pnt2splat2,
        img_shape,
        "target/del_canvas_cpu__splat_sphere__pix_sort_z.png",
    )?;
    {
        let now = std::time::Instant::now();
        del_canvas_cpu::splat_circle::draw_sort_z(
            &pnt2splat2,
            img_shape,
            "target/del_canvas_cpu__splat_sphere__sort_z.png",
        )?;
        println!(
            "   Elapsed splat circles without acceleration: {:.2?}",
            now.elapsed()
        );
    }

    // draw circles with tiles
    let now = std::time::Instant::now();
    const TILE_SIZE: usize = 16;
    let tile_shape = (
        img_shape.0 / TILE_SIZE + if img_shape.0 % TILE_SIZE == 0 {0} else {1},
        img_shape.1 / TILE_SIZE + if img_shape.0 % TILE_SIZE == 0 {0} else {1});
    let (tile2ind, ind2pnt) =
        del_canvas_cpu::tile_acceleration::tile2pnt(&pnt2splat2, img_shape, TILE_SIZE);
    //
    println!("   Elapsed tile2pnt: {:.2?}", now.elapsed());
    let now = std::time::Instant::now();
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
        for &i_pnt in &ind2pnt[tile2ind[i_tile]..tile2ind[i_tile + 1]] { // splat back to front
            let ndc_z = pnt2splat2[i_pnt].ndc_z;
            if ndc_z <= -1f32 || ndc_z >= 1f32 { continue; }
            let pos_pix = pnt2splat2[i_pnt].pos_pix;
            let rad_pix = pnt2splat2[i_pnt].rad_pix;
            let pos_pixel_center = [iw as f32 + 0.5f32, ih as f32 + 0.5f32];
            if del_geo_core::edge2::length(&pos_pix, &pos_pixel_center) > rad_pix {
                continue;
            }
            img_data[i_pix][0] = (pnt2splat3[i_pnt].rgb[0] as f32) / 255.0;
            img_data[i_pix][1] = (pnt2splat3[i_pnt].rgb[1] as f32) / 255.0;
            img_data[i_pix][2] = (pnt2splat3[i_pnt].rgb[2] as f32) / 255.0;
        }
    }
    use ::slice_of_array::SliceFlatExt; // for flat
    del_canvas_cpu::write_png_from_float_image_rgb(
        "target/del_canvas_cpu__splat_sphere__tile.png",
        &img_shape,
        (&img_data).flat(),
    )?;
    println!("   Elapsed gaussian_tile: {:.2?}", now.elapsed());
    Ok(())
}
