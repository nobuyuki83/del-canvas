use del_canvas_cuda::splat_gauss::Splat3;
use del_canvas_cuda::splat_gauss::Splat2;

fn main() -> anyhow::Result<()> {
    let file_path = "../asset/dog.ply";
    let pnt2splat3 = del_msh_core::io_ply::read_3d_gauss_splat::<_, Splat3>(file_path)?;
    // pnt2gs3.iter().enumerate().for_each(|(i_pnt, a)| { dbg!(i_pnt, a.xyz); } );
    let aabb3 = del_msh_core::vtx2point::aabb3_from_points(&pnt2splat3);
    let img_shape = (800usize+1, 1200usize+1);
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
    // --------------------------
    // below: cuda code from here
    let dev = cudarc::driver::CudaDevice::new(0)?;
    //
    let pnt2splat3_dev = dev.htod_copy(pnt2splat3.clone())?;
    let mut pnt2splat2_dev = {
        let pnt2splat2 = vec![Splat2::default(); pnt2splat3.len()];
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
    let tile_size = 16usize;
    let (tile2idx_dev, idx2pnt_dev) = del_canvas_cuda::splat_gauss::tile2idx_idx2pnt(
        &dev,
        (img_shape.0 as u32, img_shape.1 as u32), tile_size as u32,
        &pnt2splat2_dev)?;
    {
        let now = std::time::Instant::now();
        let mut pix2rgb_dev = dev.alloc_zeros::<f32>(img_shape.0 * img_shape.1 * 3)?;
        del_canvas_cuda::splat_gauss::rasterize_pnt2splat2(
            &dev, (img_shape.0 as u32, img_shape.1 as u32),
            &mut pix2rgb_dev,
            &pnt2splat2_dev,
            tile_size as u32,
            &tile2idx_dev,
            &idx2pnt_dev)?;
        println!("   Elapsed rasterize on GPU with tile: {:.2?}", now.elapsed());
        let pix2rgb = dev.dtoh_sync_copy(&pix2rgb_dev)?;
        del_canvas_cpu::write_png_from_float_image_rgb(
            "../target/del_canvas_cuda__03_splat_gauss.png",
            &img_shape,
            &pix2rgb,
        )?;

    }


    { // draw image on GPu with tile2pnt computed on cpu
        let now = std::time::Instant::now();
        let pnt2splat2 = dev.dtoh_sync_copy(&pnt2splat2_dev)?;
        let (tile2idx, idx2pnt) = del_canvas_cpu::tile_acceleration::tile2pnt::<_,u32>(&pnt2splat2, img_shape, tile_size);
        println!("num_idx: {}", idx2pnt.len());
        println!("   Elapsed tile2pnt on CPU: {:.2?}", now.elapsed());
        //
        let tile2idx_dev = dev.htod_copy(tile2idx)?;
        let idx2pnt_dev = dev.htod_copy(idx2pnt)?;
        //
        let now = std::time::Instant::now();
        let mut pix2rgb_dev = dev.alloc_zeros::<f32>(img_shape.0 * img_shape.1 * 3)?;
        del_canvas_cuda::splat_gauss::rasterize_pnt2splat2(
            &dev, (img_shape.0 as u32, img_shape.1 as u32),
            &mut pix2rgb_dev,
            &pnt2splat2_dev,
            tile_size as u32,
            &tile2idx_dev,
            &idx2pnt_dev)?;
        println!("   Elapsed rasterize on GPU with tile: {:.2?}", now.elapsed());
        let pix2rgb = dev.dtoh_sync_copy(&pix2rgb_dev)?;
        del_canvas_cpu::write_png_from_float_image_rgb(
            "../target/del_canvas_cuda__03_splat_gauss_test_rasterize.png",
            &img_shape,
            &pix2rgb,
        )?;
    }

    {
        // check tile2pnt
        let pnt2splat2 = dev.dtoh_sync_copy(&pnt2splat2_dev)?;
        let (tile2idx_cpu, idx2pnt_cpu) = del_canvas_cpu::tile_acceleration::tile2pnt::<_,u32>(
            &pnt2splat2, img_shape, tile_size);
        {
            // assert tile2idx
            let tile2idx_gpu = dev.dtoh_sync_copy(&tile2idx_dev)?;
            tile2idx_gpu
                .iter()
                .zip(tile2idx_cpu.iter())
                .for_each(|(&a, &b)| {
                    // println!("{} {}", a,b);
                    assert_eq!(a, b);
                });
        }
        {
            // assert idx2pnt
            let idx2pnt_gpu = dev.dtoh_sync_copy(&idx2pnt_dev)?;
            idx2pnt_gpu
                .iter()
                .zip(idx2pnt_cpu.iter())
                .for_each(|(&a, &b)| {
                    // println!("{} {} {} {}", a,b, pnt2splat2[a as usize].ndc_z, pnt2splat2[b as usize].ndc_z);
                    assert_eq!(a, b);
                });
        }
    }

    {   // cpu rendering from Vec<Splat2>
        let pnt2splat2 = dev.dtoh_sync_copy(&pnt2splat2_dev)?;
        println!("gaussian_naive without tile acceleration");
        let now = std::time::Instant::now();
        del_canvas_cpu::splat_gaussian2::rasterize_naive(
            &pnt2splat2,
            img_shape,
            "../target/del_canvas_cuda__03_splat_gauss_test_splat3_to_splat2.png")?;
        println!("   Elapsed gaussian_naive from Vec<Splat2>: {:.2?}", now.elapsed());
    }
    Ok(())
}