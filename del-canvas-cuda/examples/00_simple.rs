fn main() -> anyhow::Result<()> {
    let ptx = cudarc::nvrtc::compile_ptx(
        "
extern \"C\" __global__ void sin_kernel(float *out, const float *inp, const size_t numel) {
    unsigned int i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i < numel) {
        out[i] = sin(inp[i]);
    }
}",
    )?;

    let dev = cudarc::driver::CudaDevice::new(0)?;
    //
    let inp0 = dev.htod_copy(vec![1.0f32; 100])?;
    let inp1 = dev.htod_copy(vec![2.0f32; 100])?;
    let cfg = cudarc::driver::LaunchConfig::for_num_elems(100);
    use cudarc::driver::LaunchAsync;
    {
        dev.load_ptx(ptx, "my_module", &["sin_kernel"])?;
        let sin_kernel = dev.get_func("my_module", "sin_kernel").unwrap();
        let mut out = dev.alloc_zeros::<f32>(100)?;
        unsafe { sin_kernel.launch(cfg, (&mut out, &inp0, 100usize)) }?;
        let out_host: Vec<f32> = dev.dtoh_sync_copy(&out)?;
        dbg!(1f32.sin(), &out_host);
    }
    {
        dev.load_ptx(
            del_canvas_kernel_cuda::SIMPLE.into(),
            "my_module",
            &["vector_add"],
        )?;
        let vector_add = dev.get_func("my_module", "vector_add").unwrap();
        dbg!(dev.name()?);
        let mut out = dev.alloc_zeros::<f32>(100)?;
        unsafe { vector_add.launch(cfg, (&mut out, &inp0, &inp1, 100usize)) }?;
        let out_host: Vec<f32> = dev.dtoh_sync_copy(&out)?;
        dbg!(&out_host);
    }
    Ok(())
}
