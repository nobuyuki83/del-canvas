use cudarc::driver::DeviceSlice;

#[derive(Clone, Debug)]
#[repr(C)]
pub struct Splat3 {
    pub xyz: [f32; 3],
    // nrm: [f32; 3],
    pub rgb_dc: [f32; 3],
    pub rgb_sh: [f32; 45],
    pub opacity: f32,
    pub scale: [f32; 3],
    pub quaternion: [f32; 4],
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

unsafe impl cudarc::driver::DeviceRepr for Splat3 {}

// above: trait implementation for GSplat3
// ----------------------------

#[derive(Clone, Default)]
#[repr(C)]
pub struct Splat2 {
    pub pos_pix: [f32; 2],
    pub sig_inv: [f32; 3],
    pub aabb: [f32; 4],
    pub rgb: [f32; 3],
    pub alpha: f32,
    pub ndc_z: f32,
}

unsafe impl cudarc::driver::DeviceRepr for Splat2 {}

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

/*
impl del_canvas_cpu::tile_acceleration::Splat2 for Splat2{
    fn aabb(&self) -> [f32; 4] { self.aabb }
    fn ndc_z(&self) -> f32 { self.ndc_z }
}
 */

// ---------------------------------
// below: global funcs

pub fn pnt2splat3_to_pnt2splat2(
    dev: &std::sync::Arc<cudarc::driver::CudaDevice>,
    pnt2gs3_dev: &cudarc::driver::CudaSlice<Splat3>,
    pnt2gs2_dev: &mut cudarc::driver::CudaSlice<Splat2>,
    transform_world2ndc_dev: &cudarc::driver::CudaSlice<f32>,
    img_shape: (u32, u32),
) -> anyhow::Result<()> {
    let cfg = cudarc::driver::LaunchConfig::for_num_elems(pnt2gs3_dev.len() as u32);
    let param = (
        pnt2gs3_dev.len(),
        pnt2gs2_dev,
        pnt2gs3_dev,
        transform_world2ndc_dev,
        img_shape.0,
        img_shape.1,
    );
    use cudarc::driver::LaunchAsync;
    let splat3_to_splat2 = del_cudarc_util::get_or_load_func(
        &dev,
        "splat3_to_splat2",
        del_canvas_splat_cudarc_kernel::SPLAT_GAUSS,
    )?;
    unsafe { splat3_to_splat2.launch(cfg, param) }?;
    Ok(())
}

pub fn rasterize_pnt2splat2(
    dev: &std::sync::Arc<cudarc::driver::CudaDevice>,
    img_shape: (u32, u32),
    pix2rgb_dev: &mut cudarc::driver::CudaSlice<f32>,
    pnt2splat2_dev: &cudarc::driver::CudaSlice<Splat2>,
    tile_size: u32,
    tile2idx_dev: &cudarc::driver::CudaSlice<u32>,
    idx2pnt_dev: &cudarc::driver::CudaSlice<u32>,
) -> anyhow::Result<()> {
    let tile_shape = (
        img_shape.0 / tile_size + if img_shape.0 % tile_size == 0 { 0 } else { 1 },
        img_shape.1 / tile_size + if img_shape.1 % tile_size == 0 { 0 } else { 1 },
    );
    assert_eq!(
        tile2idx_dev.len(),
        (tile_shape.0 * tile_shape.1 + 1) as usize
    );
    // gpu splat
    let cfg = {
        cudarc::driver::LaunchConfig {
            grid_dim: (tile_shape.0 as u32, tile_shape.1 as u32, 1),
            block_dim: (tile_size as u32, tile_size as u32, 1),
            shared_mem_bytes: 0,
        }
    };
    let param = (
        img_shape.0 as u32,
        img_shape.1 as u32,
        pix2rgb_dev,
        tile_shape.0 as u32,
        tile_shape.1 as u32,
        tile_size as u32,
        tile2idx_dev,
        idx2pnt_dev,
        pnt2splat2_dev,
    );
    use cudarc::driver::LaunchAsync;
    let count_splat_in_tile = del_cudarc_util::get_or_load_func(
        &dev,
        "rasterize_splat_using_tile",
        del_canvas_splat_cudarc_kernel::SPLAT_GAUSS,
    )?;
    unsafe { count_splat_in_tile.launch(cfg, param) }?;
    Ok(())
}

pub fn tile2idx_idx2pnt(
    dev: &std::sync::Arc<cudarc::driver::CudaDevice>,
    img_shape: (u32, u32),
    tile_size: u32,
    pnt2splat_dev: &cudarc::driver::CudaSlice<Splat2>,
) -> anyhow::Result<(
    cudarc::driver::CudaSlice<u32>,
    cudarc::driver::CudaSlice<u32>,
)> {
    let tile_shape = (
        img_shape.0 / tile_size + if img_shape.0 % tile_size == 0 { 0 } else { 1 },
        img_shape.1 / tile_size + if img_shape.1 % tile_size == 0 { 0 } else { 1 },
    );
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
        let count_splat_in_tile = del_cudarc_util::get_or_load_func(
            &dev,
            "count_splat_in_tile",
            del_canvas_splat_cudarc_kernel::SPLAT_GAUSS,
        )?;
        unsafe { count_splat_in_tile.launch(cfg, param) }?;
        (tile2idx_dev, pnt2idx_dev)
    };
    let tile2idx_dev = {
        let mut tmp = dev.alloc_zeros(tile2idx_dev.len())?;
        del_cudarc_util::cumsum::sum_scan_blelloch(&dev, &mut tmp, &tile2idx_dev)?;
        tmp
    };
    let pnt2idx_dev = {
        let mut tmp = dev.alloc_zeros::<u32>(pnt2ind_dev.len())?;
        del_cudarc_util::cumsum::sum_scan_blelloch(&dev, &mut tmp, &pnt2ind_dev)?;
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
        let count_splat_in_tile = del_cudarc_util::get_or_load_func(
            &dev,
            "fill_index_info",
            del_canvas_splat_cudarc_kernel::SPLAT_GAUSS,
        )?;
        unsafe { count_splat_in_tile.launch(cfg, param) }?;
        del_cudarc_util::sort_by_key_u64::radix_sort_by_key_u64(
            &dev,
            &mut idx2tiledepth_dev,
            &mut idx2pnt_dev,
        )?;
        idx2pnt_dev
    };
    Ok((tile2idx_dev, idx2pnt_dev))
}
