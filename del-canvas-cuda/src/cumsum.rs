use cudarc::driver::{CudaDevice, CudaSlice, DeviceSlice};

fn block_sums(
    dev: &std::sync::Arc<CudaDevice>,
    d_out: &mut CudaSlice<u32>,
    d_in: &CudaSlice<u32>,
    num_elem: u32,
) -> anyhow::Result<(u32, u32, CudaSlice<u32>)> {
    const MAX_BLOCK_SZ: u32 = 1024;
    // const NUM_BANKS: u32 = 32;
    const LOG_NUM_BANKS: u32 = 5;

    // Zero out d_out
    dev.memset_zeros(d_out)?;

    // Set up number of threads and blocks
    let block_size = MAX_BLOCK_SZ / 2;
    let max_elems_per_block = 2 * block_size; // due to binary tree nature of algorithm

    // If input size is not power of two, the remainder will still need a whole block
    // Thus, number of blocks must be the ceiling of input size / max elems that a block can handle
    //unsigned int grid_sz = (unsigned int) std::ceil((double) numElems / (double) max_elems_per_block);
    // UPDATE: Instead of using ceiling and risking miscalculation due to precision, just automatically
    //  add 1 to the grid size when the input size cannot be divided cleanly by the block's capacity
    let mut grid_size = num_elem / max_elems_per_block;

    // Take advantage of the fact that integer division drops the decimals
    if num_elem % max_elems_per_block != 0 {
        grid_size += 1;
    }

    // Conflict free padding requires that shared memory be more than 2 * block_sz
    let shmem_size = max_elems_per_block + ((max_elems_per_block - 1) >> LOG_NUM_BANKS);

    // Allocate memory for array of total sums produced by each block
    // Array length must be the same as number of blocks
    let mut d_block_sums = dev.alloc_zeros::<u32>(grid_size as usize)?;

    // Sum scan data allocated to each block
    //gpu_sum_scan_blelloch<<<grid_sz, block_sz, sizeof(unsigned int) * max_elems_per_block >>>(d_out, d_in, d_block_sums, numElems);
    {
        let cfg = {
            cudarc::driver::LaunchConfig {
                grid_dim: (grid_size as u32, 1, 1),
                block_dim: (block_size as u32, 1, 1),
                shared_mem_bytes: (u32::BITS / 8 as u32) * shmem_size,
            }
        };
        //for_num_elems((img_size.0 * img_size.1).try_into()?);
        let param = (
            d_out,
            d_in,
            &d_block_sums,
            num_elem,
            shmem_size,
            max_elems_per_block,
        );
        use cudarc::driver::LaunchAsync;
        let gpu_prescan =
            crate::get_or_load_func(&dev, "gpu_prescan", del_canvas_kernel_cuda::CUMSUM)?;
        unsafe { gpu_prescan.launch(cfg, param) }?;
    }

    // Sum scan total sums produced by each block
    // Use basic implementation if number of total sums is <= 2 * block_sz
    //  (This requires only one block to do the scan)
    if grid_size <= max_elems_per_block {
        let d_dummy_blocks_sums = dev.alloc_zeros::<u32>(1)?;
        //gpu_sum_scan_blelloch<<<1, block_sz, sizeof(unsigned int) * max_elems_per_block>>>(d_block_sums, d_block_sums, d_dummy_blocks_sums, grid_sz);
        let cfg = {
            cudarc::driver::LaunchConfig {
                grid_dim: (1u32, 1, 1),
                block_dim: (block_size as u32, 1, 1),
                shared_mem_bytes: u32::BITS / 8u32 * shmem_size,
            }
        };
        let param = (
            &d_block_sums,
            &d_block_sums,
            &d_dummy_blocks_sums,
            grid_size,
            shmem_size,
            max_elems_per_block,
        );
        use cudarc::driver::LaunchAsync;
        let gpu_prescan =
            crate::get_or_load_func(&dev, "gpu_prescan", del_canvas_kernel_cuda::CUMSUM)?;
        unsafe { gpu_prescan.launch(cfg, param) }?;
    } else {
        // println!("prefix sum of blocks using recursive");
        // Else, recurse on this same function as you'll need the full-blown scan
        //  for the block sums
        let mut d_in_block_sums = unsafe { dev.alloc::<u32>(grid_size as usize)? };
        dev.dtod_copy(&d_block_sums, &mut d_in_block_sums)?;
        assert_eq!(d_block_sums.len(), d_in_block_sums.len());
        let (grid_sz1, block_sz1, d_block_sums1) =
            block_sums(dev, &mut d_block_sums, &d_in_block_sums, grid_size)?;
        add_block_sums(
            dev,
            &mut d_block_sums,
            &d_block_sums1,
            num_elem,
            grid_sz1,
            block_sz1,
        )?;
    }
    Ok((grid_size, block_size, d_block_sums))
}

/// Add each block's total sum to its scan output
/// in order to get the final, global scanned array
fn add_block_sums(
    dev: &std::sync::Arc<CudaDevice>,
    vout_dev: &mut CudaSlice<u32>,
    d_block_sums: &CudaSlice<u32>,
    num_elem: u32,
    grid_sz: u32,
    block_sz: u32,
) -> anyhow::Result<()> {
    let cfg = {
        cudarc::driver::LaunchConfig {
            grid_dim: (grid_sz as u32, 1, 1),
            block_dim: (block_sz as u32, 1, 1),
            shared_mem_bytes: 0,
        }
    };
    let param = (vout_dev, d_block_sums, num_elem);
    use cudarc::driver::LaunchAsync;
    let gpu_add_block_sums =
        crate::get_or_load_func(&dev, "gpu_add_block_sums", del_canvas_kernel_cuda::CUMSUM)?;
    unsafe { gpu_add_block_sums.launch(cfg, param) }?;
    Ok(())
}

/// vout_dev does not need to be zeros
pub fn sum_scan_blelloch(
    dev: &std::sync::Arc<CudaDevice>,
    vout_dev: &mut CudaSlice<u32>,
    vin_dev: &CudaSlice<u32>,
) -> anyhow::Result<()> {
    let n = vin_dev.len();
    assert_eq!(vout_dev.len(), n);
    let (grid_sz, block_sz, d_block_sums) = block_sums(&dev, vout_dev, &vin_dev, n as u32)?;
    add_block_sums(&dev, vout_dev, &d_block_sums, n as u32, grid_sz, block_sz)?;
    Ok(())
}

#[test]
fn test() -> anyhow::Result<()> {
    let dev = cudarc::driver::CudaDevice::new(0)?;
    let nvs = [
        (1024, 1024),
        (1023, 1),
        (1024, 1),
        (1025, 1),
        (1024 * 1024 - 1, 1),
        (1024 * 1024, 1),
        (1024 * 1024 + 1, 1),
        (1024 * 1024 + 1, 2),
    ];
    for (n, v) in nvs {
        let vin: Vec<u32> = {
            let mut vin = vec![v; n];
            vin.push(0u32);
            vin
        };
        let vin_dev = dev.htod_copy(vin.clone())?;
        let mut vout_dev = dev.alloc_zeros::<u32>(vin_dev.len())?;
        sum_scan_blelloch(&dev, &mut vout_dev, &vin_dev)?;
        let vout = dev.dtoh_sync_copy(&vout_dev)?;
        assert_eq!(vout.len(), n + 1);
        for i in 0..n {
            assert_eq!(
                vout[i + 1] as i64 - vout[i] as i64,
                vin[i] as i64,
                "{}-->{} {}",
                i,
                vout[i],
                vout[i + 1]
            );
        }
        assert_eq!(vout[n], (n as u32) * v);
    }
    Ok(())
}
