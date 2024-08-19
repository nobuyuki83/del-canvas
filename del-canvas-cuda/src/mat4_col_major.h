#pragma once
#include <cuda/std/array>

namespace mat4_col_major {

__device__
auto transform_homogeneous(
  const float* transform,
  const float* x) -> cuda::std::array<float,3>
{
    float y3 = transform[3] * x[0] + transform[7] * x[1] + transform[11] * x[2] + transform[15];
    const float y0 = transform[0] * x[0] + transform[4] * x[1] + transform[8] * x[2] + transform[12];
    const float y1 = transform[1] * x[0] + transform[5] * x[1] + transform[9] * x[2] + transform[13];
    const float y2 = transform[2] * x[0] + transform[6] * x[1] + transform[10] * x[2] + transform[14];
    return {y0/y3, y1/y3, y2/y3};
}

__device__
auto multmat(float* c, const float* a, const float* b) -> cuda::std::array<float,16>
{
    cuda::std::array<float,16> o;
    for(int i=0;i<4;++i) {
        for(int j=0;j<4;++j) {
           o[i + j * 4] = 0.;
            for(int k=0;k<4;++k) {
                o[i + j * 4] += a[i + k * 4] * b[k + j * 4];
            }
        }
    }
    return o;
}

}