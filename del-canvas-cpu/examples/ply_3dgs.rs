use del_geo_core::mat4_col_major::transform_homogeneous;
use itertools::Itertools;

#[derive(Clone, Debug)]
pub struct GSplat3 {
    xyz: [f32; 3],
    // nrm: [f32; 3],
    rgb_dc: [f32; 3],
    rgb_sh: [f32; 45],
    opacity: f32,
    scale: [f32; 3],
    quaternion: [f32; 4],
}

impl del_msh_core::io_ply::GaussSplat3D for GSplat3 {
    fn new(
        xyz: [f32; 3],
        rgb_dc: [f32; 3],
        rgb_sh: [f32; 45],
        opacity: f32,
        scale: [f32; 3],
        quaternion: [f32; 4],
    ) -> Self {
        GSplat3 {
            xyz,
            rgb_dc,
            rgb_sh,
            opacity,
            scale,
            quaternion,
        }
    }
}

impl del_msh_core::vtx2point::HasXyz<f32> for GSplat3 {
    fn xyz(&self) -> &[f32; 3] {
        &self.xyz
    }
}

fn draw_pix(
    pnt2gs3: &[GSplat3],
    img_shape: (usize, usize),
    transform_world2ndc: &[f32; 16],
    transform_ndc2pix: &[f32; 6],
    aabb3: &[f32; 6],
) -> anyhow::Result<()> {
    let mut img_data = vec![[0f32, 0f32, 0f32]; img_shape.0 * img_shape.1]; // black
    del_canvas_cpu::rasterize_aabb3::wireframe_dda(
        &mut img_data,
        img_shape,
        transform_world2ndc,
        &aabb3,
        [1.0, 1.0, 1.0],
    );
    for i_pnt in 0..pnt2gs3.len() {
        let q0 = transform_homogeneous(&transform_world2ndc, &pnt2gs3[i_pnt].xyz).unwrap();
        let r0 =
            del_geo_core::mat2x3_col_major::mult_vec3(&transform_ndc2pix, &[q0[0], q0[1], 1f32]);
        let ix = r0[0] as usize;
        let iy = r0[1] as usize;
        let ipix = iy * img_shape.0 + ix;
        img_data[ipix][0] = pnt2gs3[i_pnt].rgb_dc[0];
        img_data[ipix][1] = pnt2gs3[i_pnt].rgb_dc[1];
        img_data[ipix][2] = pnt2gs3[i_pnt].rgb_dc[2];
    }
    use ::slice_of_array::SliceFlatExt; // for flat
    del_canvas_cpu::write_png_from_float_image_rgb(
        "target/ply_3dgs_naive.png",
        &img_shape,
        (&img_data).flat(),
    )?;
    Ok(())
}

// above GSplat3 related funcs
// -----------------------

pub struct GSplat2 {
    pos_pix: [f32; 2],
    sig_inv: [f32; 3],
    aabb: [f32; 4],
    rgb: [f32; 3],
    ndc_z: f32,
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
    let file_path = "C:/Users/nobuy/Downloads/dog.ply";
    let pnt2gs3 = del_msh_core::io_ply::read_3d_gauss_splat::<_, GSplat3>(file_path)?;
    /*
    for i_pnt in 0..pnt2gs3.len() {
        println!("{} {:?}", i_pnt, pnt2gs3[i_pnt].scale);
    }
     */
    let aabb3 = del_msh_core::vtx2point::aabb3_from_points(&pnt2gs3);
    // dbg!(&aabb3);
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
    draw_pix(
        &pnt2gs3,
        img_shape,
        &transform_world2ndc,
        &transform_ndc2pix,
        &aabb3,
    )?;

    let mut pnt2gs2: Vec<GSplat2> = vec![];
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
        let g2 = GSplat2 {
            pos_pix,
            sig_inv,
            aabb,
            rgb: gs3.rgb_dc,
            ndc_z: ndc0[2],
        };
        pnt2gs2.push(g2);
    }

    let idx2pnt = {
        let num_pnt = pnt2gs2.len();
        let mut idx2pnt: Vec<usize> = (0..num_pnt).collect();
        idx2pnt.sort_by(|&i, &j| {
            let zi = pnt2gs2[i].ndc_z;
            let zj = pnt2gs2[j].ndc_z;
            zi.partial_cmp(&zj).unwrap()
        });
        // idx2pnt.iter().enumerate().for_each(|(idx, &i_pnt)| println!("{} {}", idx, pnt2gs2[i_pnt].ndc_z));
        idx2pnt
    };

    {
        // visualize as Gaussian without tile acceleration
        println!("gaussian_naive");
        let now = std::time::Instant::now();
        let mut img_data = vec![0f32; img_shape.1 * img_shape.0 * 3];
        for (ih, iw) in itertools::iproduct!(0..img_shape.1, 0..img_shape.0) {
            let t = [iw as f32 + 0.5, ih as f32 + 0.5];
            let mut alpha_sum = 0f32;
            let mut alpha_occu = 1f32;
            for idx in (0..idx2pnt.len()).rev() { // draw from back
                let i_pnt = idx2pnt[idx];
                let pnt2 = &pnt2gs2[i_pnt];
                // front to back
                if !del_geo_core::aabb2::is_inlcude_point(&pnt2.aabb, &[t[0], t[1]]) {
                    continue;
                }
                let t0 = [t[0] - pnt2.pos_pix[0], t[1] - pnt2.pos_pix[1]];
                let e =
                    del_geo_core::mat2_sym::mult_vec_from_both_sides(&pnt2.sig_inv, &t0, &t0);
                let e = (-0.5 * e).exp();
                let e_out = alpha_occu * e;
                img_data[(ih * img_shape.0 + iw) * 3 + 0] += pnt2.rgb[0] * e_out;
                img_data[(ih * img_shape.0 + iw) * 3 + 1] += pnt2.rgb[1] * e_out;
                img_data[(ih * img_shape.0 + iw) * 3 + 2] += pnt2.rgb[2] * e_out;
                alpha_occu *= 1f32 - e;
                alpha_sum += e_out;
                if alpha_sum > 0.999 {
                    break;
                }
            }
        }
        del_canvas_cpu::write_png_from_float_image_rgb(
            "target/ply_3dgs_gaussian.png",
            &img_shape,
            &img_data,
        )?;
        println!("   Elapsed gaussian_naive: {:.2?}", now.elapsed());
    }

    Ok(())
}
