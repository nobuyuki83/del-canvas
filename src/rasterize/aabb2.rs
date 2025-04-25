pub fn stroke_dda<Real, VAL>(
    pix2val: &mut [VAL],
    img_width: usize,
    aabb2: &[Real; 4],
    transform_world2pix: &[Real; 9],
    val: VAL,
) where
    Real: num_traits::Float + num_traits::AsPrimitive<usize> + std::fmt::Debug,
    usize: num_traits::AsPrimitive<Real>,
    VAL: Copy,
{
    // let img_height = pix2val.len() / img_width;
    use del_geo_core::mat3_col_major::transform_homogeneous;
    let q0: [Real; 2] = transform_homogeneous(transform_world2pix, &[aabb2[0], aabb2[1]]).unwrap();
    let q1: [Real; 2] = transform_homogeneous(transform_world2pix, &[aabb2[2], aabb2[1]]).unwrap();
    let q2: [Real; 2] = transform_homogeneous(transform_world2pix, &[aabb2[2], aabb2[3]]).unwrap();
    let q3: [Real; 2] = transform_homogeneous(transform_world2pix, &[aabb2[0], aabb2[3]]).unwrap();
    crate::rasterize::line2::draw_dda_pixel_coordinate(pix2val, img_width, &q0, &q1, val);
    crate::rasterize::line2::draw_dda_pixel_coordinate(pix2val, img_width, &q1, &q2, val);
    crate::rasterize::line2::draw_dda_pixel_coordinate(pix2val, img_width, &q2, &q3, val);
    crate::rasterize::line2::draw_dda_pixel_coordinate(pix2val, img_width, &q3, &q0, val);
}
