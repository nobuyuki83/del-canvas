use num_traits::AsPrimitive;

pub fn paint_one_pixel<Real, VAL>(
    pix2color: &mut [VAL],
    img_width: usize,
    p0: &[Real; 2],
    transform_xy2pix: &[Real; 9],
    i_color: VAL,
) where
    Real: num_traits::Float + AsPrimitive<usize>,
{
    let img_height = pix2color.len() / img_width;
    let q0: [Real; 2] =
        del_geo_core::mat3_col_major::transform_homogeneous(transform_xy2pix, p0).unwrap();
    let iw: usize = q0[0].floor().as_();
    let ih: usize = q0[1].floor().as_();
    if iw >= img_width || ih >= img_height {
        return;
    }
    pix2color[ih * img_width + iw] = i_color;
}
