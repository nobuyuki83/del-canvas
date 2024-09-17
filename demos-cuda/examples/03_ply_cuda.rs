use cudarc::driver::{DevicePtrMut, DeviceRepr, DeviceSlice};
use num_traits::AsPrimitive;

#[derive(Clone, Default)]
#[repr(C)]
pub struct XyzRgb {
    xyz: [f32;3],
    rgb: [u8;3]
}

impl del_msh_core::io_ply::Xyz for XyzRgb{
    fn set_xyz(&mut self, xyz: &[f64; 3]) {
        self.xyz = xyz.map(|v| v as f32);
    }
}

impl del_msh_core::io_ply::Rgb for XyzRgb{
    fn set_rgb(&mut self, rgb: &[u8; 3]) {
        self.rgb = *rgb;
    }
}

impl del_msh_core::vtx2xyz::HasXyz<f32> for XyzRgb {
    fn xyz(&self) -> [f32; 3] {
        self.xyz
    }
}
unsafe impl cudarc::driver::DeviceRepr for XyzRgb {}

#[derive(Clone, Default, Debug)]
#[repr(C)]
struct Splat {
    ndc_z: f32,
    pos_pix: [f32; 2],
    rad_pix: f32,
}

unsafe impl cudarc::driver::DeviceRepr for Splat {}



fn main() -> anyhow::Result<()> {
    // let path = "/Users/nobuyuki/project/juice_box1.ply";
    let file_path = "C:/Users/nobuy/Downloads/juice_box.ply";
    let pnt2xyzrgb = del_msh_core::io_ply::read_xyzrgb::<_, XyzRgb>(file_path)?;
    let aabb3 = del_msh_core::vtx2xyz::aabb3_from_points(&pnt2xyzrgb);
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
    let radius = 0.0015f32;
    let pnt2splat = vec!(Splat::default(); pnt2xyzrgb.len());

    let dev = cudarc::driver::CudaDevice::new(0)?;
    dev.load_ptx(
        del_canvas_kernel_cuda::XYZRGB.into(),
        "xyzrgb",
        &["xyzrgb_to_splat", "count_splat_in_tile", "fill_index_info"])?;
    dev.load_ptx(
        del_canvas_kernel_cuda::CUMSUM.into(),
        "cumsum",
        &["gpu_prescan", "gpu_add_block_sums"])?;

    let pnt2xyzrgb_dev = dev.htod_copy(pnt2xyzrgb.clone())?;
    let mut pnt2splat_dev = dev.htod_copy(pnt2splat.clone())?;
    let transform_world2ndc_dev = dev.htod_copy(transform_world2ndc.to_vec())?;
    {
        let cfg = cudarc::driver::LaunchConfig::for_num_elems(pnt2xyzrgb.len() as u32);
        let param = (
            pnt2xyzrgb_dev.len(),
            &mut pnt2splat_dev,
            &pnt2xyzrgb_dev,
            &transform_world2ndc_dev,
            img_shape.0 as u32,
            img_shape.1 as u32,
            radius
        );
        use cudarc::driver::LaunchAsync;
        let xyzrgb_to_splat = dev.get_func("xyzrgb", "xyzrgb_to_splat").unwrap();
        unsafe { xyzrgb_to_splat.launch(cfg, param) }?;
    }

    {
        // draw pixels in cpu
        let pnt2splat = dev.dtoh_sync_copy(&pnt2splat_dev)?;
        let idx2vtx = {
            let mut idx2vtx: Vec<usize> = (0..pnt2splat.len()).collect();
            idx2vtx.sort_by(|&idx0, &idx1| {
                pnt2splat[idx0].ndc_z.partial_cmp(&pnt2splat[idx1].ndc_z).unwrap()
            });
            idx2vtx
        };
        let mut img_data = vec![[0f32, 0f32, 0f32]; img_shape.0 * img_shape.1];
        del_canvas_core::rasterize_aabb3::wireframe_dda(
            &mut img_data,
            img_shape, &transform_world2ndc, &aabb3, [1.0, 1.0, 1.0]);
        for i_idx in 0..pnt2xyzrgb.len() {
            let i_vtx = idx2vtx[i_idx];
            let r0 = pnt2splat[i_vtx].pos_pix;
            let ix = r0[0] as usize;
            let iy = r0[1] as usize;
            let ipix = iy * img_shape.0 + ix;
            img_data[ipix][0] = (pnt2xyzrgb[i_vtx].rgb[0] as f32) / 255.0;
            img_data[ipix][1] = (pnt2xyzrgb[i_vtx].rgb[1] as f32) / 255.0;
            img_data[ipix][2] = (pnt2xyzrgb[i_vtx].rgb[2] as f32) / 255.0;
        }
        use ::slice_of_array::SliceFlatExt; // for flat
        del_canvas_core::write_png_from_float_image_rgb(
            "../target/ply_pixel_cuda.png",
            &img_shape,
            (&img_data).flat(),
        )?;
    } // end pixel
    // ---------------------------------------------------------
    {
        // draw circles with tiles
        let now = std::time::Instant::now();
        const TILE_SIZE: usize = 16;
        assert_eq!(img_shape.0 % TILE_SIZE, 0);
        assert_eq!(img_shape.0 % TILE_SIZE, 0);
        let tile_shape = (
            img_shape.0 / TILE_SIZE,
            img_shape.1 / TILE_SIZE
        );
        let (tile2ind_dev, pnt2ind_dev) = {
            let num_pnt = pnt2splat_dev.len();
            let mut tile2idx_dev = dev.alloc_zeros::<u32>(tile_shape.0 * tile_shape.1 + 1)?;
            let mut pnt2idx_dev = dev.alloc_zeros::<u32>(num_pnt + 1)?;
            let cfg = cudarc::driver::LaunchConfig::for_num_elems(num_pnt as u32);
            let param = (
                pnt2splat_dev.len(),
                &pnt2splat_dev,
                &mut tile2idx_dev,
                &mut pnt2idx_dev,
                tile_shape.0 as u32,
                tile_shape.1 as u32,
                16u32
            );
            use cudarc::driver::LaunchAsync;
            let count_splat_in_tile = dev.get_func("xyzrgb", "count_splat_in_tile").unwrap();
            unsafe { count_splat_in_tile.launch(cfg, param) }?;
            (tile2idx_dev, pnt2idx_dev)
        };
        let tile2idx_dev = demos_cuda::cumsum::sum_scan_blelloch(&dev, &tile2ind_dev)?;
        //
        println!("   Elapsed tile2idx: {:.2?}", now.elapsed());
        {
            // debug tile2ind
            let pnt2splat = dev.dtoh_sync_copy(&pnt2splat_dev)?;
            let num_tile = tile_shape.0 * tile_shape.1;
            let mut tile2idx0 = vec!(0usize; num_tile + 1);
            for i_vtx in 0..pnt2splat.len() {
                let p0 = pnt2splat[i_vtx].pos_pix;
                let rad = pnt2splat[i_vtx].rad_pix;
                let aabb2 = del_geo_core::aabb2::from_point(&p0, rad);
                let tiles = del_geo_core::aabb2::overlapping_tiles(&aabb2, TILE_SIZE, tile_shape);
                for &i_tile in tiles.iter() {
                    tile2idx0[i_tile + 1] += 1;
                }
            }
            for i_tile in 0..num_tile {
                tile2idx0[i_tile + 1] += tile2idx0[i_tile];
            }
            let tile2idx = dev.dtoh_sync_copy(&tile2idx_dev)?;
            tile2idx.iter().zip(tile2idx0.iter()).for_each(|(&a, &b)| {assert_eq!(a as usize, b);} );
        } // debug tile2ind
        let pnt2idx_dev = demos_cuda::cumsum::sum_scan_blelloch(&dev, &pnt2ind_dev)?;
        let num_ind = dev.dtoh_sync_copy(&pnt2idx_dev)?.last().unwrap().to_owned(); // todo: send only last element to cpu
        debug_assert_eq!(num_ind, dev.dtoh_sync_copy(&tile2idx_dev)?.last().unwrap().to_owned());
        dbg!(num_ind);
        {
            let mut idx2tiledepth_dev = dev.alloc_zeros::<u64>(num_ind as usize)?;
            let num_pnt = pnt2splat_dev.len();
            let cfg = cudarc::driver::LaunchConfig::for_num_elems(num_pnt as u32);
            let param = (
                pnt2splat_dev.len(),
                &pnt2splat_dev,
                &pnt2idx_dev,
                &mut idx2tiledepth_dev,
                tile_shape.0 as u32,
                tile_shape.1 as u32,
                16u32
            );
            use cudarc::driver::LaunchAsync;
            let count_splat_in_tile = dev.get_func("xyzrgb", "fill_index_info").unwrap();
            unsafe { count_splat_in_tile.launch(cfg, param) }?;
            // idx2tiledepth_dev
            // dbg!( dev.dtoh_sync_copy(&idx2tiledepth_dev) );
            let a = idx2tiledepth_dev.device_ptr_mut();
        };

    } // end circle splat

    Ok(())
}


/*
let cfg = {
    cudarc::driver::LaunchConfig {
        grid_dim: (
            (img_shape.0 / TILE_SIZE + 1) as u32,
            (img_shape.1 / TILE_SIZE + 1) as u32,
            1),
        block_dim: (
            TILE_SIZE as u32,
            TILE_SIZE as u32,
            1),
        shared_mem_bytes: 0
    }
};
 */