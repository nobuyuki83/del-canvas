pub mod cumsum;
pub mod splat_sphere;
pub mod sort_by_key_u64;
pub mod sort_u32;
pub mod sort_u64;
pub mod splat_gauss;
pub mod pix2tri;

pub fn get_or_load_func(
    dev: &std::sync::Arc<cudarc::driver::CudaDevice>,
    module_name: &str,
    ptx: &'static str,
) -> anyhow::Result<cudarc::driver::CudaFunction> {
    if !dev.has_func(module_name, module_name) {
        // Leaking the string here is a bit sad but we need a &'static str and this is only
        // done once per kernel name.
        let static_module_name = Box::leak(module_name.to_string().into_boxed_str());
        dev.load_ptx(ptx.into(), module_name, &[static_module_name])?
    }
    Ok(dev.get_func(module_name, module_name).unwrap())
}
