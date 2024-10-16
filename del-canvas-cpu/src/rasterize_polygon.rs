use crate::rasterize_line;
use num_traits::AsPrimitive;
use std::ops::AddAssign;

/// * `transform` - 3x3 homogeneous transformation matrix with **column major** order
#[allow(clippy::identity_op)]
pub fn stroke<T, VAL>(
    img_data: &mut [VAL],
    width: usize,
    vtx2xy: &[T],
    transform_xy2pix: &[T; 9],
    thickness: T,
    color: VAL,
) where
    T: num_traits::Float + nalgebra::RealField + num_traits::AsPrimitive<usize>,
    usize: AsPrimitive<T>,
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
            transform_xy2pix,
            thickness,
            color,
        );
    }
}

#[allow(clippy::identity_op)]
pub fn fill<T, VAL>(
    img_data: &mut [VAL],
    width: usize,
    vtx2xy: &[T],
    transform_xy2pix: &[T; 9],
    color: VAL,
) where
    T: num_traits::Float
        + num_traits::FloatConst
        + num_traits::AsPrimitive<usize>
        + AddAssign
        + std::fmt::Debug,
    usize: AsPrimitive<T>,
    VAL: Copy,
{
    let transform_pix2xy = del_geo_core::mat3_col_major::try_inverse(transform_xy2pix).unwrap();
    let half = T::one() / (T::one() + T::one());
    let height = img_data.len() / width;
    let aabbi = {
        let aabb = del_msh_core::vtx2xy::aabb2(vtx2xy);
        let aabb = del_geo_core::aabb2::transform_homogeneous(&aabb, transform_xy2pix);
        del_geo_core::aabb2::rasterize(&aabb, &(width, height))
    };
    for ih in aabbi[1]..aabbi[3] {
        for iw in aabbi[0]..aabbi[2] {
            let w: T = iw.as_() + half; // pixel center
            let h: T = ih.as_() + half; // pixel center
            let p = del_geo_core::mat3_col_major::transform_homogeneous(&transform_pix2xy, &[w, h])
                .unwrap();
            let wn = del_msh_core::polyloop2::winding_number_(vtx2xy, &p);
            if (wn - T::one()).round() == T::zero() {
                img_data[ih * width + iw] = color;
            }
        }
    }
}

#[test]
fn test0() -> anyhow::Result<()> {
    let img_size = (100usize, 100usize);
    let trans_world2pix =
        crate::cam2::transform_world2pix_ortho_preserve_asp(&img_size, &[-0.1, -0.1, 1.1, 1.1]);
    let mut img_data = vec![0f32; img_size.0 * img_size.1];
    fill(
        &mut img_data,
        img_size.0,
        &[0.0, 0.0, 1.0, 0.0, 1.0, 0.2, 0.2, 0.3, 1.0, 1.0, 0.0, 1.0],
        &trans_world2pix,
        1f32,
    );
    crate::write_png_from_float_image_grayscale(
        "../target/rasterize_polygon-test0.png",
        &img_size,
        &img_data,
    )?;
    Ok(())
}
