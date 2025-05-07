use num_traits::AsPrimitive;

/// # Argument
/// * `transform` - 3x3 homogeneous transformation matrix with **column major** order
pub fn stroke_dda<T, VAL>(
    img_data: &mut [VAL],
    width: usize,
    vtx2xy: &[[T; 2]],
    transform_xy2pix: &[T; 9],
    color: VAL,
) where
    T: num_traits::Float + num_traits::AsPrimitive<usize> + std::fmt::Debug,
    usize: AsPrimitive<T>,
    VAL: Copy,
{
    for vtx in vtx2xy.windows(2) {
        let p0 = &vtx[0];
        let p1 = &vtx[1];
        crate::rasterize::line2::draw_dda(img_data, width, p0, p1, transform_xy2pix, color);
    }
}
