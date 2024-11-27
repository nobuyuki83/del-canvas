namespace tile_acceleration {

__device__ uint32_t float_to_uint32(float value) {
    uint32_t result;
    memcpy(&result, &value, sizeof(result));
    return result;
}

__device__ uint64_t concatenate32To64(uint32_t a, uint32_t b) {
    return ((uint64_t)b) | (((uint64_t)a) << 32);
}

__device__
void fill_index_info(
  int i_pnt,
  const cuda::std::array<float,4>& aabb,
  float ndc_z,
  const uint32_t* pnt2idx,
  uint64_t* idx2tiledepth,
  uint32_t* idx2pnt,
  uint32_t tile_w,
  uint32_t tile_h,
  uint32_t tile_size)
{
    float tile_size_f = float(tile_size);
    int ix0 = int(floor(aabb[0] / tile_size_f));
    int iy0 = int(floor(aabb[1] / tile_size_f));
    int ix1 = int(floor(aabb[2] / tile_size_f))+1;
    int iy1 = int(floor(aabb[3] / tile_size_f))+1;
    uint32_t cnt = 0;
    // printf("%d %d %d %d\n", ix0, iy0, ix1, iy1);
    for(int ix = ix0; ix < ix1; ++ix ) {
        if( ix < 0 || ix >= tile_w ){
            continue;
        }
        for(int iy=iy0;iy<iy1;++iy) {
            if( iy < 0 || iy >= tile_h ){
                continue;
            }
            uint32_t i_tile = iy * tile_w + ix;
            float zp1 = ndc_z + 1.f;
            if( zp1 <= 0.f ){ zp1 = 0.f; }  // radix sort of float cannot handle negative value
            uint32_t zi = float_to_uint32(zp1);
            uint64_t tiledepth= concatenate32To64(i_tile, zi);
            idx2tiledepth[pnt2idx[i_pnt] + cnt] = tiledepth;
            idx2pnt[pnt2idx[i_pnt] + cnt] = i_pnt;
            ++cnt;
        }
    }
}

__device__
void count_splat_in_tile(
  int i_pnt,
  const cuda::std::array<float,4>& aabb,
  uint32_t* tile2ind,
  uint32_t* pnt2ind,
  uint32_t tile_w,
  uint32_t tile_h,
  uint32_t tile_size)
{
    float tile_size_f = float(tile_size);
    int ix0 = int(floor(aabb[0] / tile_size_f));
    int iy0 = int(floor(aabb[1] / tile_size_f));
    int ix1 = int(floor(aabb[2] / tile_size_f))+1;
    int iy1 = int(floor(aabb[3] / tile_size_f))+1;
    uint32_t cnt = 0;
    // printf("%d %d %d %d\n", ix0, iy0, ix1, iy1);
    for(int ix = ix0; ix < ix1; ++ix ) {
        if( ix < 0 || ix >= tile_w ){
            continue;
        }
        for(int iy=iy0;iy<iy1;++iy) {
            if( iy < 0 || iy >= tile_h ){
                continue;
            }
            int i_tile = iy * tile_w + ix;
            // printf("%d %d\n", i_pnt, i_tile);
            atomicAdd(&tile2ind[i_tile], 1);
            ++cnt;
        }
    }
    pnt2ind[i_pnt] = cnt;
}


} // namespace tile_acceleration