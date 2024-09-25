use del_geo_core::mat4_col_major::transform_homogeneous;

#[derive(Clone, Debug)]
pub struct Splat3 {
    xyz: [f32; 3],
    // nrm: [f32; 3],
    rgb_dc: [f32; 3],
    rgb_sh: [f32; 45],
    opacity: f32,
    scale: [f32; 3],
    quaternion: [f32; 4],
}

impl del_msh_core::io_ply::GaussSplat3D for Splat3 {
    fn new(
        xyz: [f32; 3],
        rgb_dc: [f32; 3],
        rgb_sh: [f32; 45],
        opacity: f32,
        scale: [f32; 3],
        quaternion: [f32; 4],
    ) -> Self {
        Splat3 {
            xyz,
            rgb_dc,
            rgb_sh,
            opacity,
            scale,
            quaternion,
        }
    }
}

impl del_msh_core::vtx2point::HasXyz<f32> for Splat3 {
    fn xyz(&self) -> &[f32; 3] {
        &self.xyz
    }
}

impl del_canvas_cpu::splat_point3::Splat3 for Splat3 {
    fn rgb(&self) -> [f32; 3] {
        self.rgb_dc
    }
    fn xyz(&self) -> [f32; 3] {
        self.xyz
    }
}

// above GSplat3 related funcs
// -----------------------

pub struct Splat2 {
    pos_pix: [f32; 2],
    sig_inv: [f32; 3],
    aabb: [f32; 4],
    rgb: [f32; 3],
    ndc_z: f32,
}

impl del_canvas_cpu::splat_gaussian2z::Splat2 for Splat2 {
    fn ndc_z(&self) -> f32 {
        self.ndc_z
    }
    fn aabb(&self) -> &[f32; 4] {
        &self.aabb
    }
    fn property(&self) -> (&[f32; 2], &[f32; 3], &[f32; 3]) {
        (&self.pos_pix, &self.sig_inv, &self.rgb)
    }
}

fn world2pix(
    pos_world: &[f32; 3],
    transform_world2ndc: &[f32; 16],
    img_shape: (usize, usize),
) -> [f32; 6] {
    let mvp_grad =
        del_geo_core::mat4_col_major::jacobian_transform(&transform_world2ndc, &pos_world);
    let ndc2pix = del_geo_core::mat2x3_col_major::transform_ndc2pix(img_shape);
    let world2pix = del_geo_core::mat2x3_col_major::mult_mat3_col_major(&ndc2pix, &mvp_grad);
    world2pix
}

fn main() -> anyhow::Result<()> {
    let file_path = "asset/dog.ply";
    let pnt2gs3 = del_msh_core::io_ply::read_3d_gauss_splat::<_, Splat3>(file_path)?;
    let aabb3 = del_msh_core::vtx2point::aabb3_from_points(&pnt2gs3);
    let img_shape = (600usize, 1000usize);
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
    del_canvas_cpu::splat_point3::draw_pix(
        &pnt2gs3,
        img_shape,
        &transform_world2ndc,
        "target/del_canvas_cpu__splat_gauss__pix.png",
    )?;

    let mut pnt2gs2: Vec<Splat2> = vec![];
    for i_pnt in 0..pnt2gs3.len() {
        let gs3 = &pnt2gs3[i_pnt];
        let ndc0 =
            del_geo_core::mat4_col_major::transform_homogeneous(&transform_world2ndc, &gs3.xyz)
                .unwrap();
        let pos_pix =
            del_geo_core::mat2x3_col_major::mult_vec3(&transform_ndc2pix, &[ndc0[0], ndc0[1], 1.0]);
        let transform_world2pix = world2pix(&gs3.xyz, &transform_world2ndc, img_shape);
        let (abc, _dabcdt) = del_geo_core::mat2_sym::wdw_projected_spd_mat3(
            &transform_world2pix,
            &gs3.quaternion,
            &gs3.scale,
        );
        let sig_inv = del_geo_core::mat2_sym::safe_inverse_preserve_positive_definiteness::<f32>(
            &abc, 1.0e-5f32,
        );
        let aabb = del_geo_core::mat2_sym::aabb2(&sig_inv);
        let aabb = del_geo_core::aabb2::scale(&aabb, 3.0);
        let aabb = del_geo_core::aabb2::translate(&aabb, &pos_pix);
        let g2 = Splat2 {
            pos_pix,
            sig_inv,
            aabb,
            rgb: gs3.rgb_dc,
            ndc_z: ndc0[2],
        };
        pnt2gs2.push(g2);
    }

    {
        println!("gaussian_naive");
        let now = std::time::Instant::now();
        del_canvas_cpu::splat_gaussian2z::rasterize_naive(
            &pnt2gs2,
            img_shape,
            "target/ply_3dgs_gaussian.png",
        )?;
        println!("   Elapsed gaussian_naive: {:.2?}", now.elapsed());
    }

    Ok(())
}
