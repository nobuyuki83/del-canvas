
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
    alpha: f32
}

impl del_canvas_cpu::splat_gaussian2::Splat2 for Splat2 {
    fn ndc_z(&self) -> f32 {
        self.ndc_z
    }
    fn aabb(&self) -> &[f32; 4] {
        &self.aabb
    }
    fn property(&self) -> (&[f32; 2], &[f32; 3], &[f32; 3], f32) {
        (&self.pos_pix, &self.sig_inv, &self.rgb, self.alpha)
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
    let file_path = "C:/Users/nobuy/Downloads/ChilliPepperPlant.ply"; //"asset/dog.ply";
    let pnt2splat3 = {
        let mut pnt2splat3 = del_msh_core::io_ply::read_3d_gauss_splat::<_, Splat3>(file_path)?;
        let aabb3 = del_msh_core::vtx2point::aabb3_from_points(&pnt2splat3);
        let longest_edge = del_geo_core::aabb3::max_edge_size(&aabb3);
        let scale = 1.0 / longest_edge;
        let scale_sqrt = scale * scale;
        let center = del_geo_core::aabb3::center(&aabb3);
        pnt2splat3.iter_mut().for_each(|s| {
            s.xyz[0] -= center[0];
            s.xyz[1] -= center[1];
            s.xyz[2] -= center[2];
            s.xyz[0] *= scale;
            s.xyz[1] *= scale;
            s.xyz[2] *= scale;
            s.scale[0] *= scale_sqrt;
            s.scale[1] *= scale_sqrt;
            s.scale[2] *= scale_sqrt;
        });
        pnt2splat3
    };
    let img_shape = (600usize + 1, 1000usize + 1);
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
                0f32,
                0f32,
                1.4f32,
            ],
            0f32,
            0f32,
            0f32,
        );
        del_geo_core::mat4_col_major::mult_mat(&cam_proj, &cam_modelview)
    };
    let transform_ndc2pix = del_geo_core::mat2x3_col_major::transform_ndc2pix(img_shape);
    del_canvas_cpu::splat_point3::draw_pix(
        &pnt2splat3,
        img_shape,
        &transform_world2ndc,
        "target/del_canvas_cpu__splat_gauss__pix.png",
    )?;

    let mut pnt2splat2 = Vec::< Splat2 >::with_capacity(pnt2splat3.len());
    for i_pnt in 0..pnt2splat3.len() {
        let gs3 = &pnt2splat3[i_pnt];
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
            alpha: gs3.opacity
        };
        pnt2splat2.push(g2);
    }

    {
        println!("gaussian_naive");
        let now = std::time::Instant::now();
        del_canvas_cpu::splat_gaussian2::rasterize_naive(
            &pnt2splat2,
            img_shape,
            "target/del_canvas_cpu__splat_gauss_ply__naive.png",
        )?;
        println!("   Elapsed gaussian_naive: {:.2?}", now.elapsed());
    }

    Ok(())
}
