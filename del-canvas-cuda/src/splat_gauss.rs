use cudarc::driver::DeviceSlice;
use crate::splat_sphere::{Splat2, Splat3};

#[derive(Clone, Debug)]
#[repr(C)]
pub struct GSplat3 {
    pub xyz: [f32; 3],
    // nrm: [f32; 3],
    pub rgb_dc: [f32; 3],
    pub rgb_sh: [f32; 45],
    pub opacity: f32,
    pub scale: [f32; 3],
    pub quaternion: [f32; 4],
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

unsafe impl cudarc::driver::DeviceRepr for GSplat3 {}

// above: trait implementation for GSplat3
// ----------------------------

#[derive(Clone, Default)]
#[repr(C)]
pub struct GSplat2 {
    pub pos_pix: [f32; 2],
    pub sig_inv: [f32; 3],
    pub aabb: [f32; 4],
    pub rgb: [f32; 3],
    pub ndc_z: f32,
}

unsafe impl cudarc::driver::DeviceRepr for GSplat2 {}

// ---------------------------------
// below: global funcs

pub fn pnt2splat3_to_pnt2splat2(
    dev: &std::sync::Arc<cudarc::driver::CudaDevice>,
    pnt2gs3_dev: &cudarc::driver::CudaSlice<GSplat3>,
    pnt2gs2_dev: &mut cudarc::driver::CudaSlice<GSplat2>,
    transform_world2ndc_dev: &cudarc::driver::CudaSlice<f32>,
    img_shape: (u32, u32)
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
    let splat3_to_splat2 =
        crate::get_or_load_func(&dev, "splat3_to_splat2", del_canvas_cuda_kernel::SPLAT_GAUSS)?;
    unsafe { splat3_to_splat2.launch(cfg, param) }?;
    Ok(())
}

