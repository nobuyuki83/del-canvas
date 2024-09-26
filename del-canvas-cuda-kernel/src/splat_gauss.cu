#include "mat2_sym.h"
#include "mat4_col_major.h"
#include "mat2x3_col_major.h"
#include "quaternion.h"
#include "aabb2.h"

extern "C" {

struct Splat3 {
    float xyz[3];
    float rgb_dc[3];
    float rgb_sh[45];
    float opacity;
    float scale[3];
    float quaternion[4];
};

struct Splat2 {
    float pos_pix[2];
    float sig_inv[3];
    float aabb[4];
    float rgb[3];
    float alpha;
    float ndc_z;
};

__global__
void splat3_to_splat2(
  uint32_t num_pnt,
  Splat2* pnt2splat2,
  const Splat3 *pnt2splat3,
  const float *transform_world2ndc,
  const uint32_t img_w,
  const uint32_t img_h)
{
    int i_pnt = blockDim.x * blockIdx.x + threadIdx.x;
    if( i_pnt >= num_pnt ){ return; }
    //
    const auto pos_world = pnt2splat3[i_pnt].xyz;
    const cuda::std::array<float,9> world2ndc = mat4_col_major::jacobian_transform(transform_world2ndc, pos_world);
    const cuda::std::array<float,6> ndc2pix = mat2x3_col_major::transform_ndc2pix(img_w, img_h);
    const cuda::std::array<float,6> world2pix = mat2x3_col_major::mult_mat3_col_major(ndc2pix.data(), world2ndc.data());
    const auto pos_ndc = mat4_col_major::transform_homogeneous(
        transform_world2ndc, pos_world);
    const float pos_scrn[3] = {pos_ndc[0], pos_ndc[1], 1.f};
    const auto pos_pix = mat2x3_col_major::mult_vec3(ndc2pix.data(), pos_scrn);
    const cuda::std::array<float,3> sig = mat2_sym::projected_spd_mat3(
        world2pix.data(),
        pnt2splat3[i_pnt].quaternion,
        pnt2splat3[i_pnt].scale);
    const cuda::std::array<float,3> sig_inv = mat2_sym::safe_inverse_preserve_positive_definiteness(sig.data(), 1.0e-5f);
    const cuda::std::array<float,4> _aabb0 = mat2_sym::aabb2(sig_inv.data());
    const cuda::std::array<float,4> _aabb1 = aabb2::scale(_aabb0.data(), 3.f);
    const cuda::std::array<float,4> aabb = aabb2::translate(_aabb1.data(), pos_pix.data());
    //
    pnt2splat2[i_pnt].ndc_z = pos_ndc[2];
    pnt2splat2[i_pnt].pos_pix[0] = pos_pix[0];
    pnt2splat2[i_pnt].pos_pix[1] = pos_pix[1];
    pnt2splat2[i_pnt].sig_inv[0] = sig_inv[0];
    pnt2splat2[i_pnt].sig_inv[1] = sig_inv[1];
    pnt2splat2[i_pnt].sig_inv[2] = sig_inv[2];
    pnt2splat2[i_pnt].aabb[0] = aabb[0];
    pnt2splat2[i_pnt].aabb[1] = aabb[1];
    pnt2splat2[i_pnt].aabb[2] = aabb[2];
    pnt2splat2[i_pnt].aabb[3] = aabb[3];
    pnt2splat2[i_pnt].alpha = pnt2splat3[i_pnt].opacity;
    pnt2splat2[i_pnt].rgb[0] = pnt2splat3[i_pnt].rgb_dc[0];
    pnt2splat2[i_pnt].rgb[1] = pnt2splat3[i_pnt].rgb_dc[1];
    pnt2splat2[i_pnt].rgb[2] = pnt2splat3[i_pnt].rgb_dc[2];
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
    // const uint32_t i_pix = iy * img_w + ix;
    //
    const uint32_t i_tile = (iy / tile_size) * tile_w + (ix / tile_size);
    const float t[2] = {float(ix) + 0.5f, float(iy) + 0.5f};
    float alpha_sum = 0.f;
    float alpha_occu = 1.f;
    // for(int32_t idx=int32_t(d_tile2idx[i_tile+1])-1; idx>=d_tile2idx[i_tile]; --idx){
    // for(uint32_t idx=d_tile2idx[i_tile]; idx; ++idx){
    const uint32_t num_pnt = d_tile2idx[i_tile+1] - d_tile2idx[i_tile];
    for (uint32_t iidx=0;iidx<num_pnt;++iidx) {
        uint32_t idx = d_tile2idx[i_tile] + num_pnt - 1 - iidx;
        const uint32_t i_pnt = d_idx2pnt[idx];
        const Splat2& pnt2 = d_pnt2splat[i_pnt];
        // front to back
        if( !aabb2::is_inlcude_point(pnt2.aabb, t) ){
            continue;
        }
        const float t0[2] = {t[0] - pnt2.pos_pix[0], t[1] - pnt2.pos_pix[1]};
        float _e = mat2_sym::mult_vec_from_both_sides(pnt2.sig_inv, t0, t0);
        float e = expf(-0.5 * _e) * pnt2.alpha;
        float e_out = alpha_occu * e;
        d_pix2rgb[(iy * img_w + ix) * 3 + 0] += pnt2.rgb[0] * e_out;
        d_pix2rgb[(iy * img_w + ix) * 3 + 1] += pnt2.rgb[1] * e_out;
        d_pix2rgb[(iy * img_w + ix) * 3 + 2] += pnt2.rgb[2] * e_out;
        alpha_occu *= 1.f - e;
        alpha_sum += e_out;
        if( alpha_sum > 0.999 ){
            break;
        }
    }
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
    const float* aabb = splat.aabb;
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
    const float* aabb = splat.aabb;
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
            float zp1 = splat.ndc_z + 1.f;
            if( zp1 <= 0.f ){ zp1 = 0.f; }  // radix sort of float cannot handle negative value
            uint32_t zi = float_to_uint32(zp1);
            uint64_t tiledepth= concatenate32To64(i_tile, zi);
            idx2tiledepth[pnt2idx[i_pnt] + cnt] = tiledepth;
            idx2pnt[pnt2idx[i_pnt] + cnt] = i_pnt;
            ++cnt;
        }
    }
    // pnt2ind[i_pnt] = cnt;
}



}