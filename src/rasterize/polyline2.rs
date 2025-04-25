use num_traits::AsPrimitive;

/// # Argument
/// * `transform` - 3x3 homogeneous transformation matrix with **column major** order
pub fn stroke_dda<T, VAL>(
    img_data: &mut [VAL],
    width: usize,
    vtx2xy: &[T],
    transform_xy2pix: &[T; 9],
    color: VAL,
) where
    T: num_traits::Float + num_traits::AsPrimitive<usize> + std::fmt::Debug,
    usize: AsPrimitive<T>,
    VAL: Copy,
{
    let n = vtx2xy.len() / 2;
    for i in 0..n - 1 {
        let j = i + 1;
        crate::rasterize::line2::draw_dda(
            img_data,
            width,
            &[vtx2xy[i * 2], vtx2xy[i * 2 + 1]],
            &[vtx2xy[j * 2], vtx2xy[j * 2 + 1]],
            transform_xy2pix,
            color,
        );
    }
}
