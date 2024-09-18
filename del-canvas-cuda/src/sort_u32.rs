use cudarc::driver::{CudaDevice, CudaSlice, DeviceSlice};

// An attempt at the gpu radix sort variant described in this paper:
// https://vgc.poly.edu/~csilva/papers/cgf.pdf
fn radix_sort_u32(
    dev: &std::sync::Arc<CudaDevice>,
    d_in: &mut CudaSlice<u32>,
) -> anyhow::Result<()> {
    let d_in_len = d_in.len() as u32;
    const MAX_BLOCK_SZ: u32 = 128;
    let block_sz: u32 = MAX_BLOCK_SZ;
    let max_elems_per_block = block_sz;
    let grid_sz: u32 = {
        let mut grid_sz = d_in_len as u32 / max_elems_per_block;
        // Take advantage of the fact that integer division drops the decimals
        if d_in_len % max_elems_per_block != 0 {
            grid_sz += 1;
        }
        grid_sz
    };

    let mut d_out = dev.alloc_zeros::<u32>(d_in.len())?;
    let mut d_prefix_sums = dev.alloc_zeros::<u32>(d_in_len as usize)?;

    // let d_block_sums;
    let d_block_sums_len = 4 * grid_sz; // 4-way split
    let mut d_block_sums = dev.alloc_zeros::<u32>(d_block_sums_len as usize)?;
    let mut d_scan_block_sums = dev.alloc_zeros::<u32>(d_block_sums_len as usize)?;

    // shared memory consists of 3 arrays the size of the block-wise input
    //  and 2 arrays the size of n in the current n-way split (4)
    let s_data_len = max_elems_per_block;
    let s_mask_out_len = max_elems_per_block + 1;
    let s_merged_scan_mask_out_len = max_elems_per_block;
    let s_mask_out_sums_len = 4; // 4-way split
    let s_scan_mask_out_sums_len = 4;
    let shmem_sz = s_data_len
        + s_mask_out_len
        + s_merged_scan_mask_out_len
        + s_mask_out_sums_len
        + s_scan_mask_out_sums_len;

    // for every 2 bits from LSB to MSB:
    //  block-wise radix sort (write blocks back to global memory)
    for shift_width in (0..=30).step_by(2) {
        {
            let cfg = cudarc::driver::LaunchConfig {
                grid_dim: (grid_sz, 1, 1),
                block_dim: (block_sz, 1, 1),
                shared_mem_bytes: shmem_sz * (u32::BITS / 8),
            };
            gpu_radix_sort_local(
                dev,
                cfg,
                &mut d_out,
                &mut d_prefix_sums,
                &mut d_block_sums,
                shift_width,
                d_in,
                max_elems_per_block,
            )?;
        }

        // scan global block sum array
        crate::cumsum::sum_scan_blelloch(dev, &mut d_scan_block_sums, &d_block_sums)?;

        glbl_shuffle(
            dev,
            grid_sz,
            block_sz,
            d_in,
            &mut d_out,
            &d_scan_block_sums,
            &d_prefix_sums,
            shift_width,
            max_elems_per_block,
        )?
    }
    // dev.dtod_copy(d_in, d_out)?;
    Ok(())
}

fn gpu_radix_sort_local(
    dev: &std::sync::Arc<CudaDevice>,
    cfg: cudarc::driver::LaunchConfig,
    d_out: &mut CudaSlice<u32>,
    d_prefix_sums: &mut CudaSlice<u32>,
    d_block_sums: &mut CudaSlice<u32>,
    shift_width: u32,
    d_in: &mut CudaSlice<u32>,
    max_elems_per_block: u32,
) -> anyhow::Result<()> {
    let d_in_len = d_in.len() as u32;
    let param = (
        d_out,
        d_prefix_sums,
        d_block_sums,
        shift_width,
        d_in,
        d_in_len,
        max_elems_per_block,
    );
    use cudarc::driver::LaunchAsync;
    let gpu_radix_sort_local = crate::get_or_load_func(
        &dev,
        "gpu_radix_sort_local",
        del_canvas_cuda_kernel::SORT_U32,
    )?;
    unsafe { gpu_radix_sort_local.launch(cfg, param) }?;
    Ok(())
}

fn glbl_shuffle(
    dev: &std::sync::Arc<CudaDevice>,
    grid_sz: u32,
    block_sz: u32,
    d_in: &mut CudaSlice<u32>,
    d_out: &CudaSlice<u32>,
    d_scan_block_sums: &CudaSlice<u32>,
    d_prefix_sums: &CudaSlice<u32>,
    shift_width: u32,
    max_elems_per_block: u32,
) -> anyhow::Result<()> {
    // scatter/shuffle block-wise sorted array to final positions
    let d_in_len = d_in.len() as u32;
    let cfg = cudarc::driver::LaunchConfig {
        grid_dim: (grid_sz, 1, 1),
        block_dim: (block_sz, 1, 1),
        shared_mem_bytes: 0,
    };
    let param = (
        d_in,
        d_out,
        d_scan_block_sums,
        d_prefix_sums,
        shift_width,
        d_in_len,
        max_elems_per_block,
    );
    use cudarc::driver::LaunchAsync;
    let gpu_glbl_shuffle =
        crate::get_or_load_func(&dev, "gpu_glbl_shuffle", del_canvas_cuda_kernel::SORT_U32)?;
    unsafe { gpu_glbl_shuffle.launch(cfg, param) }?;
    Ok(())
}

#[test]
fn test_u32() -> anyhow::Result<()> {
    let dev = cudarc::driver::CudaDevice::new(0)?;
    let ns = [
        13usize,
        1023,
        1024,
        1024 * 1024 - 1,
        1024 * 1024,
        1024 * 1024 + 1,
    ];
    // let ns = [13usize];
    use rand::Rng;
    use rand_chacha::rand_core::SeedableRng;
    for n in ns {
        let mut rng = rand_chacha::ChaChaRng::from_seed([0; 32]);
        let vin = {
            let mut vin: Vec<u32> = vec![];
            (0..n).for_each(|_| vin.push(rng.gen()));
            vin
        };
        let mut vio_dev = dev.htod_copy::<u32>(vin.clone())?;
        // let mut vout_dev = dev.alloc_zeros::<u32>(vin_dev.len())?;
        radix_sort_u32(&dev, &mut vio_dev)?;
        // dbg!( dev.dtoh_sync_copy(&vin_dev));
        let vout0 = {
            // naive cpu computation
            let mut vout0 = vin.clone();
            vout0.sort();
            vout0
        };
        let vout = dev.dtoh_sync_copy(&vio_dev)?;
        vout.iter().zip(vout0.iter()).for_each(|(a, b)| {
            assert_eq!(a, b);
        });
    }
    Ok(())
}
