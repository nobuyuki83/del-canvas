#define MAX_BLOCK_SZ 128

extern "C" {

__global__ void gpu_set_consecutive_sequence(
    uint32_t* d_out,
    uint32_t num_d_out)
{
    int i_d_out = blockDim.x * blockIdx.x + threadIdx.x;
    if( i_d_out >= num_d_out ){ return; }
    d_out[i_d_out] = i_d_out;
}

__global__ void gpu_radix_sort_local(
    uint64_t* d_out_sorted,
    uint32_t* d_prefix_sums,
    uint32_t* d_block_sums,
    unsigned int input_shift_width,
    uint64_t* d_in,
    unsigned int d_in_len,
    unsigned int max_elems_per_block,
    const uint32_t* d_in_index,
    uint32_t* d_out_index)
{
    // need shared memory array for:
    // - block's share of the input data (local sort will be put here too)
    // - mask outputs
    // - scanned mask outputs
    // - merged scaned mask outputs ("local prefix sum")
    // - local sums of scanned mask outputs
    // - scanned local sums of scanned mask outputs

    // for all radix combinations:
    //  build mask output for current radix combination
    //  scan mask output
    //  store needed value from current prefix sum array to merged prefix sum array
    //  store total sum of mask output (obtained from scan) to global block sum array
    // calculate local sorted address from local prefix sum and scanned mask output's total sums
    // shuffle input block according to calculated local sorted addresses
    // shuffle local prefix sums according to calculated local sorted addresses
    // copy locally sorted array back to global memory
    // copy local prefix sum array back to global memory

    extern __shared__ unsigned int shmem[];
    uint64_t* s_data = (uint64_t*)shmem;
    // s_mask_out[] will be scanned in place
    unsigned int* s_mask_out = &shmem[max_elems_per_block*2];
    unsigned int s_mask_out_len = max_elems_per_block + 1;
    unsigned int* s_merged_scan_mask_out = &s_mask_out[s_mask_out_len];
    unsigned int* s_mask_out_sums = &s_merged_scan_mask_out[max_elems_per_block];
    unsigned int* s_scan_mask_out_sums = &s_mask_out_sums[4];

    unsigned int thid = threadIdx.x;

    // Copy block's portion of global input data to shared memory
    unsigned int cpy_idx = max_elems_per_block * blockIdx.x + thid;
    if (cpy_idx < d_in_len)
        s_data[thid] = d_in[cpy_idx];
    else
        s_data[thid] = 0;

    __syncthreads();

    // To extract the correct 2 bits, we first shift the number
    //  to the right until the correct 2 bits are in the 2 LSBs,
    //  then mask on the number with 11 (3) to remove the bits
    //  on the left
    const uint64_t t_data = s_data[thid];
    unsigned int t_2bit_extract = (t_data >> input_shift_width) & 3;

    for (unsigned int i = 0; i < 4; ++i)
    {
        // Zero out s_mask_out
        s_mask_out[thid] = 0;
        if (thid == 0)
            s_mask_out[s_mask_out_len - 1] = 0;

        __syncthreads();

        // build bit mask output
        bool val_equals_i = false;
        if (cpy_idx < d_in_len)
        {
            val_equals_i = t_2bit_extract == i;
            s_mask_out[thid] = val_equals_i;
        }
        __syncthreads();

        // Scan mask outputs (Hillis-Steele)
        int partner = 0;
        unsigned int sum = 0;
        unsigned int max_steps = (unsigned int) log2f(max_elems_per_block);
        for (unsigned int d = 0; d < max_steps; d++) {
            partner = thid - (1 << d);
            if (partner >= 0) {
                sum = s_mask_out[thid] + s_mask_out[partner];
            }
            else {
                sum = s_mask_out[thid];
            }
            __syncthreads();
            s_mask_out[thid] = sum;
            __syncthreads();
        }

        // Shift elements to produce the same effect as exclusive scan
        unsigned int cpy_val = 0;
        cpy_val = s_mask_out[thid];
        __syncthreads();
        s_mask_out[thid + 1] = cpy_val;
        __syncthreads();

        if (thid == 0)
        {
            // Zero out first element to produce the same effect as exclusive scan
            s_mask_out[0] = 0;
            unsigned int total_sum = s_mask_out[s_mask_out_len - 1];
            s_mask_out_sums[i] = total_sum;
            d_block_sums[i * gridDim.x + blockIdx.x] = total_sum;
        }
        __syncthreads();

        if (val_equals_i && (cpy_idx < d_in_len))
        {
            s_merged_scan_mask_out[thid] = s_mask_out[thid];
        }

        __syncthreads();
    }

    // Scan mask output sums
    // Just do a naive scan since the array is really small
    if (thid == 0)
    {
        unsigned int run_sum = 0;
        for (unsigned int i = 0; i < 4; ++i)
        {
            s_scan_mask_out_sums[i] = run_sum;
            run_sum += s_mask_out_sums[i];
        }
    }

    __syncthreads();

    if (cpy_idx < d_in_len)
    {
        // Calculate the new indices of the input elements for sorting
        unsigned int t_prefix_sum = s_merged_scan_mask_out[thid];
        unsigned int new_pos = t_prefix_sum + s_scan_mask_out_sums[t_2bit_extract];

        __syncthreads();

        // Shuffle the block's input elements to actually sort them
        // Do this step for greater global memory transfer coalescing
        //  in next step
        s_data[new_pos] = t_data;
        s_merged_scan_mask_out[new_pos] = t_prefix_sum;

        __syncthreads();

        // Copy block - wise prefix sum results to global memory
        // Copy block-wise sort results to global
        d_prefix_sums[cpy_idx] = s_merged_scan_mask_out[thid];
        d_out_sorted[cpy_idx] = s_data[thid];
        d_out_index[max_elems_per_block * blockIdx.x + new_pos] = d_in_index[cpy_idx];
    }
}

__global__ void gpu_glbl_shuffle(
    uint64_t* d_out,
    const uint64_t* d_in,
    const unsigned int* d_scan_block_sums,
    const unsigned int* d_prefix_sums,
    unsigned int input_shift_width,
    unsigned int d_in_len,
    unsigned int max_elems_per_block,
    const uint32_t* d_in_index,
    uint32_t* d_out_index)
{
    // get d = digit
    // get n = blockIdx
    // get m = local prefix sum array value
    // calculate global position = P_d[n] + m
    // copy input element to final position in d_out

    unsigned int thid = threadIdx.x;
    unsigned int cpy_idx = max_elems_per_block * blockIdx.x + thid;

    if (cpy_idx < d_in_len)
    {
        uint64_t t_data = d_in[cpy_idx];
        unsigned int t_2bit_extract = (t_data >> input_shift_width) & 3;
        unsigned int t_prefix_sum = d_prefix_sums[cpy_idx];
        unsigned int data_glbl_pos = d_scan_block_sums[t_2bit_extract * gridDim.x + blockIdx.x]
            + t_prefix_sum;
        __syncthreads();
        d_out[data_glbl_pos] = t_data;
        d_out_index[data_glbl_pos] = d_in_index[cpy_idx];
    }
}

}