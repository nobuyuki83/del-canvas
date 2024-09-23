use cudarc::driver::DeviceSlice;

#[derive(Clone, Default)]
#[repr(C)]
pub struct Splat3 {
    pub xyz: [f32; 3],
    pub rgb: [u8; 3],
}

impl del_msh_core::io_ply::XyzRgb for Splat3 {
    fn new(xyz: [f64; 3], rgb: [u8; 3]) -> Self {
        Splat3 {
            xyz: xyz.map(|v| v as f32),
            rgb
        }
    }
}

impl del_msh_core::vtx2point::HasXyz<f32> for Splat3 {
    fn xyz(&self) -> &[f32; 3] {
        &self.xyz
    }
}
unsafe impl cudarc::driver::DeviceRepr for Splat3 {}

#[derive(Clone, Default, Debug)]
#[repr(C)]
pub struct Splat2 {
    pub ndc_z: f32,
    pub pos_pix: [f32; 2],
    pub rad_pix: f32,
    pub rgb: [f32; 3],
}

unsafe impl cudarc::driver::DeviceRepr for Splat2 {}

pub fn tile2idx_idx2pnt(
    dev: &std::sync::Arc<cudarc::driver::CudaDevice>,
    tile_shape: (u32, u32),
    pnt2splat_dev: &cudarc::driver::CudaSlice<Splat2>,
) -> anyhow::Result<(
    cudarc::driver::CudaSlice<u32>,
    cudarc::driver::CudaSlice<u32>,
)> {
    let (tile2idx_dev, pnt2ind_dev) = {
        let num_pnt = pnt2splat_dev.len();
        let mut tile2idx_dev =
            dev.alloc_zeros::<u32>((tile_shape.0 * tile_shape.1 + 1) as usize)?;
        let mut pnt2idx_dev = dev.alloc_zeros::<u32>(num_pnt + 1)?;
        let cfg = cudarc::driver::LaunchConfig::for_num_elems(num_pnt as u32);
        let param = (
            pnt2splat_dev.len(),
            pnt2splat_dev,
            &mut tile2idx_dev,
            &mut pnt2idx_dev,
            tile_shape.0 as u32,
            tile_shape.1 as u32,
            16u32,
        );
        use cudarc::driver::LaunchAsync;
        let count_splat_in_tile =
            crate::get_or_load_func(&dev, "count_splat_in_tile", del_canvas_cuda_kernel::SPLAT_SPHERE)?;
        unsafe { count_splat_in_tile.launch(cfg, param) }?;
        (tile2idx_dev, pnt2idx_dev)
    };
    let tile2idx_dev = {
        let mut tmp = dev.alloc_zeros(tile2idx_dev.len())?;
        crate::cumsum::sum_scan_blelloch(&dev, &mut tmp, &tile2idx_dev)?;
        tmp
    };
    let pnt2idx_dev = {
        let mut tmp = dev.alloc_zeros::<u32>(pnt2ind_dev.len())?;
        crate::cumsum::sum_scan_blelloch(&dev, &mut tmp, &pnt2ind_dev)?;
        tmp
    };
    let num_ind = dev.dtoh_sync_copy(&pnt2idx_dev)?.last().unwrap().to_owned(); // todo: send only last element to cpu
    debug_assert_eq!(
        num_ind,
        dev.dtoh_sync_copy(&tile2idx_dev)?
            .last()
            .unwrap()
            .to_owned()
    );
    let idx2pnt_dev = {
        let mut idx2tiledepth_dev = dev.alloc_zeros::<u64>(num_ind as usize)?;
        let mut idx2pnt_dev = dev.alloc_zeros(num_ind as usize)?;
        let num_pnt = pnt2splat_dev.len();
        let cfg = cudarc::driver::LaunchConfig::for_num_elems(num_pnt as u32);
        let param = (
            pnt2splat_dev.len(),
            pnt2splat_dev,
            &pnt2idx_dev,
            &mut idx2tiledepth_dev,
            &mut idx2pnt_dev,
            tile_shape.0 as u32,
            tile_shape.1 as u32,
            16u32,
        );
        use cudarc::driver::LaunchAsync;
        let count_splat_in_tile =
            crate::get_or_load_func(&dev, "fill_index_info", del_canvas_cuda_kernel::SPLAT_SPHERE)?;
        unsafe { count_splat_in_tile.launch(cfg, param) }?;
        crate::sort_by_key_u64::radix_sort_by_key_u64(
            &dev,
            &mut idx2tiledepth_dev,
            &mut idx2pnt_dev,
        )?;
        idx2pnt_dev
    };
    Ok((tile2idx_dev, idx2pnt_dev))
}

pub fn pnt2splat3_to_pnt2splat2(
    dev: &std::sync::Arc<cudarc::driver::CudaDevice>,
    pnt2splat3_dev: &cudarc::driver::CudaSlice<Splat3>,
    pnt2splat2_dev: &mut cudarc::driver::CudaSlice<Splat2>,
    transform_world2ndc_dev: &cudarc::driver::CudaSlice<f32>,
    img_shape: (u32, u32),
    radius: f32,
) -> anyhow::Result<()> {
    let cfg = cudarc::driver::LaunchConfig::for_num_elems(pnt2splat3_dev.len() as u32);
    let param = (
        pnt2splat3_dev.len(),
        pnt2splat2_dev,
        pnt2splat3_dev,
        transform_world2ndc_dev,
        img_shape.0,
        img_shape.1,
        radius,
    );
    use cudarc::driver::LaunchAsync;
    let xyzrgb_to_splat =
        crate::get_or_load_func(&dev, "splat3_to_splat2", del_canvas_cuda_kernel::SPLAT_SPHERE)?;
    unsafe { xyzrgb_to_splat.launch(cfg, param) }?;
    Ok(())
}

pub fn splat(
    dev: &std::sync::Arc<cudarc::driver::CudaDevice>,
    img_shape: (u32, u32),
    pix2rgb_dev: &mut cudarc::driver::CudaSlice<f32>,
    pnt2splat_dev: &cudarc::driver::CudaSlice<Splat2>,
    TILE_SIZE: u32,
    tile2idx_dev: &cudarc::driver::CudaSlice<u32>,
    idx2pnt_dev: &cudarc::driver::CudaSlice<u32>,
) -> anyhow::Result<()> {
    assert_eq!(img_shape.0 % TILE_SIZE, 0);
    assert_eq!(img_shape.0 % TILE_SIZE, 0);
    let tile_shape = (img_shape.0 / TILE_SIZE, img_shape.1 / TILE_SIZE);
    // gpu splat
    let cfg = {
        cudarc::driver::LaunchConfig {
            grid_dim: (
                (img_shape.0 / TILE_SIZE + 1) as u32,
                (img_shape.1 / TILE_SIZE + 1) as u32,
                1,
            ),
            block_dim: (TILE_SIZE as u32, TILE_SIZE as u32, 1),
            shared_mem_bytes: 0,
        }
    };
    let param = (
        img_shape.0 as u32,
        img_shape.1 as u32,
        pix2rgb_dev,
        tile_shape.0 as u32,
        tile_shape.1 as u32,
        TILE_SIZE as u32,
        tile2idx_dev,
        idx2pnt_dev,
        pnt2splat_dev,
    );
    use cudarc::driver::LaunchAsync;
    let count_splat_in_tile = crate::get_or_load_func(
        &dev,
        "rasterize_splat_using_tile",
        del_canvas_cuda_kernel::SPLAT_SPHERE)?;
    unsafe { count_splat_in_tile.launch(cfg, param) }?;
    Ok(())
}
