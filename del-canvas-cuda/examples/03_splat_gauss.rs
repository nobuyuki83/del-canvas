use del_canvas_cuda::splat_gauss::GSplat3;
use del_canvas_cuda::splat_gauss::GSplat2;

fn main() -> anyhow::Result<()> {
    let file_path = "C:/Users/nobuy/Downloads/dog.ply";
    let pnt2gs3 = del_msh_core::io_ply::read_3d_gauss_splat::<_, GSplat3>(file_path)?;
    // pnt2gs3.iter().enumerate().for_each(|(i_pnt, a)| { dbg!(i_pnt, a.xyz); } );
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
    // --------------------------
    // below: cuda code from here
    let dev = cudarc::driver::CudaDevice::new(0)?;
    //
    let pnt2splat3_dev = dev.htod_copy(pnt2gs3.clone())?;
    let mut pnt2splat2_dev = {
        let pnt2splat2 = vec![GSplat2::default(); pnt2gs3.len()];
        dev.htod_copy(pnt2splat2.clone())?
    };
    let transform_world2ndc_dev = dev.htod_copy(transform_world2ndc.to_vec())?;
    del_canvas_cuda::splat_gauss::pnt2splat3_to_pnt2splat2(
        &dev,
        &pnt2splat3_dev,
        &mut pnt2splat2_dev,
        &transform_world2ndc_dev,
        (img_shape.0 as u32, img_shape.1 as u32),
    )?;

    {
        let pnt2splat2 = dev.dtoh_sync_copy(&pnt2splat2_dev)?;
        let idx2pnt = {
            let num_pnt = pnt2splat2.len();
            let mut idx2pnt: Vec<usize> = (0..num_pnt).collect();
            idx2pnt.sort_by(|&i, &j| {
                let zi = pnt2splat2[i].ndc_z;
                let zj = pnt2splat2[j].ndc_z;
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
                    let pnt2 = &pnt2splat2[i_pnt];
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
                "../target/03_splat_gauss_test_splat3_to_splat2.png",
                &img_shape,
                &img_data,
            )?;
            println!("   Elapsed gaussian_naive: {:.2?}", now.elapsed());
        }
    }

    Ok(())
}