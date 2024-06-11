use crate::rasterize_line;
use num_traits::AsPrimitive;

/// * `transform` - 3x3 homogeneous transformation matrix with **column major** order
#[allow(clippy::identity_op)]
pub fn stroke<T, VAL>(
    img_data: &mut [VAL],
    width: usize,
    vtx2xy: &[T],
    transform: &[T;9],
    point_size: T,
    color: VAL,
) where
    T: num_traits::Float + nalgebra::RealField + num_traits::AsPrimitive<i64>,
    i64: AsPrimitive<T>,
    VAL: Copy,
{
    let n = vtx2xy.len() / 2;
    for i in 0..n {
        let j = (i + 1) % n;
        rasterize_line::draw_pixcenter(
            img_data,
            width,
            &[vtx2xy[i * 2 + 0], vtx2xy[i * 2 + 1]],
            &[vtx2xy[j * 2 + 0], vtx2xy[j * 2 + 1]],
            transform,
            point_size,
            color,
        );
    }
}
