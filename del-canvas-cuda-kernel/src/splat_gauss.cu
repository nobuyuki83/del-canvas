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
    // printf("%d %lf %lf %lf\n", i_pnt, sig_inv[0], sig_inv[1], sig_inv[2]);
    // printf("%d %f %f %f %f\n", i_pnt, aabb[0], aabb[1], aabb[2], aabb[3]);
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
    pnt2splat2[i_pnt].rgb[0] = pnt2splat3[i_pnt].rgb_dc[0];
    pnt2splat2[i_pnt].rgb[1] = pnt2splat3[i_pnt].rgb_dc[1];
    pnt2splat2[i_pnt].rgb[2] = pnt2splat3[i_pnt].rgb_dc[2];
/*

   pnt2splat[i_pnt].rad = rad;
   pnt2splat[i_pnt].rgb[0] = float(pnt2xyzrgb[i_pnt].rgb[0]) / 255.0;
   pnt2splat[i_pnt].rgb[1] = float(pnt2xyzrgb[i_pnt].rgb[1]) / 255.0;
   pnt2splat[i_pnt].rgb[2] = float(pnt2xyzrgb[i_pnt].rgb[2]) / 255.0;
*/
}


}