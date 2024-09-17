#include <cub/cub.cuh>
#include "mat4_col_major.h"
#include "aabb2.h"

extern "C" {

struct XyzRgb{
    float xyz[3];
    unsigned char rgb[3];
};

struct Splat {
    float z;
    float pos_pix[2];
    float rad;
};

__global__
void xyzrgb_to_splat(
  uint32_t num_pnt,
  Splat* pnt2splat,
  const XyzRgb *pnt2xyzrgb,
  const float *transform_world2ndc,
  const uint32_t img_w,
  const uint32_t img_h,
  float radius)
{
    int i_pnt = blockDim.x * blockIdx.x + threadIdx.x;
    if( i_pnt >= num_pnt ){ return; }
    //
    const auto p0 = pnt2xyzrgb[i_pnt].xyz;
    const auto q0 = mat4_col_major::transform_homogeneous(
        transform_world2ndc, p0);
   float r0[2] = {
     (q0[0] + 1.f) * 0.5f * float(img_w),
     (1.f - q0[1]) * 0.5f * float(img_h) };
   float rad;
   {
       const cuda::std::array<float,9> dqdp = mat4_col_major::jacobian_transform(transform_world2ndc, p0);
       const cuda::std::array<float,9> dpdq = mat3_col_major::try_inverse(dqdp.data()).value();
       const float dx[3] = { dpdq[0], dpdq[1], dpdq[2] };
       const float dy[3] = { dpdq[3], dpdq[4], dpdq[5] };
       float rad_pix_x = (1.f / vec3::norm(dx)) * 0.5f * float(img_w) * radius;
       float rad_pxi_y = (1.f / vec3::norm(dy)) * 0.5f * float(img_h) * radius;
       rad = 0.5f * (rad_pix_x + rad_pxi_y);
   }
   pnt2splat[i_pnt].z = q0[2];
   pnt2splat[i_pnt].pos_pix[0] = r0[0];
   pnt2splat[i_pnt].pos_pix[1] = r0[1];
   pnt2splat[i_pnt].rad = rad;
}


__global__
void count_splat_in_tile(
  uint32_t num_pnt,
  const Splat* pnt2splat,
  uint32_t* tile2ind,
  uint32_t* pnt2ind,
  uint32_t tile_w,
  uint32_t tile_h,
  uint32_t tile_size)
{
    int i_pnt = blockDim.x * blockIdx.x + threadIdx.x;
    if( i_pnt >= num_pnt ){ return; }
    //
    const Splat& splat = pnt2splat[i_pnt];
    const cuda::std::array<float,4> aabb = aabb2::from_point(splat.pos_pix, splat.rad);
    //
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

__device__ uint32_t float_to_uint32(float value) {
    uint32_t result;
    memcpy(&result, &value, sizeof(result));
    return result;
}

__device__ uint64_t concatenate32To64(uint32_t a, uint32_t b) {
    // b を64ビットの下位部分に、a を64ビットの上位部分にシフトして結合
    return ((uint64_t)b) | (((uint64_t)a) << 32);
}

__global__
void fill_index_info(
  uint32_t num_pnt,
  const Splat* pnt2splat,
  const uint32_t* pnt2idx,
  uint64_t* idx2tiledepth,
  uint32_t tile_w,
  uint32_t tile_h,
  uint32_t tile_size)
{
    int i_pnt = blockDim.x * blockIdx.x + threadIdx.x;
    if( i_pnt >= num_pnt ){ return; }
    //
    const Splat& splat = pnt2splat[i_pnt];
    const cuda::std::array<float,4> aabb = aabb2::from_point(splat.pos_pix, splat.rad);
    //
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
            uint32_t depth_in_uint32 = float_to_uint32(splat.z);
            uint64_t tiledepth= concatenate32To64(i_tile, depth_in_uint32);
            idx2tiledepth[pnt2idx[i_pnt] + cnt] = tiledepth;
            ++cnt;
        }
    }
    // pnt2ind[i_pnt] = cnt;
}



} // extern "C"