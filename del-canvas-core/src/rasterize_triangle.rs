use num_traits::AsPrimitive;

#[allow(clippy::identity_op)]
pub fn fill<Index, Real, VAL>(
    pix2color: &mut [VAL],
    img_width: usize,
    p0: &[Real; 2],
    p1: &[Real; 2],
    p2: &[Real; 2],
    transform_xy2pix: &[Real; 9],
    i_color: VAL,
) where
    Real: num_traits::Float + 'static + Copy + AsPrimitive<usize>,
    Index: AsPrimitive<usize>,
    usize: AsPrimitive<Real>,
    VAL: Copy,
{
    let half = Real::one() / (Real::one() + Real::one());
    let img_height = pix2color.len() / img_width;
    let q0: [Real; 2] =
        del_geo_core::mat3_col_major::transform_homogeneous(transform_xy2pix, p0).unwrap();
    let q1: [Real; 2] =
        del_geo_core::mat3_col_major::transform_homogeneous(transform_xy2pix, p1).unwrap();
    let q2: [Real; 2] =
        del_geo_core::mat3_col_major::transform_homogeneous(transform_xy2pix, p2).unwrap();
    let aabbi = {
        let aabb = del_msh_core::vtx2xy::aabb2(&[q0[0], q0[1], q1[0], q1[1], q2[0], q2[1]]);
        del_geo_core::aabb2::rasterize(&aabb, &(img_width, img_height))
    };
    for i_h in aabbi[1]..aabbi[3] {
        for i_w in aabbi[0]..aabbi[2] {
            let p_xy: [Real; 2] = [i_w.as_() + half, i_h.as_() + half];
            let Some((_r0, _r1)) =
                del_geo_core::tri2::is_inside(&q0, &q1, &q2, &p_xy, -Real::one())
            else {
                continue;
            };
            pix2color[i_h * img_width + i_w] = i_color;
        }
    }
}

#[test]
fn test0() {
    let img_size = (100usize, 100usize);
    let trans_xy2pix =
        crate::cam2::transform_world2pix_ortho_preserve_asp(&img_size, &[-0.1, -0.1, 1.1, 1.1]);
    let mut img_data = vec![0f32; img_size.0 * img_size.1];
    fill::<usize, f32, f32>(
        &mut img_data,
        img_size.0,
        &[0.1, 0.1],
        &[0.5, 0.1],
        &[0.9, 0.8],
        &trans_xy2pix,
        1f32,
    );
    crate::write_png_from_float_image_grayscale(
        "target/rasterize_triangle-test0.png",
        &img_size,
        &img_data,
    );
}
