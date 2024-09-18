#include <cuda/std/array>

namespace aabb2 {

__device__
auto from_point(
    const float* p,
    float rad) -> cuda::std::array<float,4>
{
    return {
        p[0] - rad,
        p[1] - rad,
        p[0] + rad,
        p[1] + rad};
}

}