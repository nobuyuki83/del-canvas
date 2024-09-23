#include "mat4_col_major.h"
#include "aabb2.h"

extern "C" {

struct Splat3{
    float xyz[3];
    unsigned char rgb[3];
};

struct Splat2 {
    float z;
    float pos_pix[2];
    float rad;
    float rgb[3];
};

__global__
void splat3_to_splat2(
  uint32_t num_pnt,
  Splat2* pnt2splat,
  const Splat3 *pnt2xyzrgb,
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
   pnt2splat[i_pnt].rgb[0] = float(pnt2xyzrgb[i_pnt].rgb[0]) / 255.0;
   pnt2splat[i_pnt].rgb[1] = float(pnt2xyzrgb[i_pnt].rgb[1]) / 255.0;
   pnt2splat[i_pnt].rgb[2] = float(pnt2xyzrgb[i_pnt].rgb[2]) / 255.0;
}


__global__
void count_splat_in_tile(
  uint32_t num_pnt,
  const Splat2* pnt2splat,
  uint32_t* tile2ind,
  uint32_t* pnt2ind,
  uint32_t tile_w,
  uint32_t tile_h,
  uint32_t tile_size)
{
    int i_pnt = blockDim.x * blockIdx.x + threadIdx.x;
    if( i_pnt >= num_pnt ){ return; }
    //
    const Splat2& splat = pnt2splat[i_pnt];
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
    return ((uint64_t)b) | (((uint64_t)a) << 32);
}

__global__
void fill_index_info(
  uint32_t num_pnt,
  const Splat2* pnt2splat,
  const uint32_t* pnt2idx,
  uint64_t* idx2tiledepth,
  uint32_t* idx2pnt,
  uint32_t tile_w,
  uint32_t tile_h,
  uint32_t tile_size)
{
    int i_pnt = blockDim.x * blockIdx.x + threadIdx.x;
    if( i_pnt >= num_pnt ){ return; }
    //
    const Splat2& splat = pnt2splat[i_pnt];
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
            uint32_t zi = float_to_uint32(splat.z);
            {  // making the negative float value comparable
                zi &= ~(1 << 31); // set zero to 31st bit
                zi = ~zi; // invert bit
            }
            uint64_t tiledepth= concatenate32To64(i_tile, zi);
            idx2tiledepth[pnt2idx[i_pnt] + cnt] = tiledepth;
            idx2pnt[pnt2idx[i_pnt] + cnt] = i_pnt;
            ++cnt;
        }
    }
    // pnt2ind[i_pnt] = cnt;
}

__global__
void rasterize_splat_using_tile(
    uint32_t img_w,
    uint32_t img_h,
    float* d_pix2rgb,
    uint32_t tile_w,
    uint32_t tile_h,
    uint32_t tile_size,
    const uint32_t* d_tile2idx,
    const uint32_t* d_idx2pnt,
    const Splat2* d_pnt2splat)
{
    const uint32_t ix = blockDim.x * blockIdx.x + threadIdx.x;
    if( ix >= img_w ){ return; }
    //
    const uint32_t iy = blockDim.y * blockIdx.y + threadIdx.y;
    if( iy >= img_h ){ return; }
    const uint32_t i_pix = iy * img_w + ix;
    //
    const uint32_t i_tile = (iy / tile_size) * tile_w + (ix / tile_size);
    for(uint32_t idx=d_tile2idx[i_tile]; idx<d_tile2idx[i_tile+1];++idx){
        const uint32_t i_pnt = d_idx2pnt[idx];
        const Splat2& splat = d_pnt2splat[i_pnt];
        const float p0[2] = {
            float(ix) + 0.5f,
            float(iy) + 0.5f};
        const float dx = splat.pos_pix[0] - p0[0];
        const float dy = splat.pos_pix[1] - p0[1];
        const float distance = sqrt(dx * dx + dy * dy);
        if( distance > splat.rad ){ continue; }
        d_pix2rgb[i_pix*3+0] = splat.rgb[0];
        d_pix2rgb[i_pix*3+1] = splat.rgb[1];
        d_pix2rgb[i_pix*3+2] = splat.rgb[2];
    }

}


} // extern "C"