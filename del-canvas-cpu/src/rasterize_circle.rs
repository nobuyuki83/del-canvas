use num_traits::AsPrimitive;

pub fn pixels_in_point<Real>(x: Real, y: Real, rad: Real, width: usize, height: usize) -> Vec<usize>
where
    Real: num_traits::Float + 'static + AsPrimitive<i64>,
    i64: AsPrimitive<Real>,
{
    let half: Real = Real::one() / (Real::one() + Real::one());
    let iwmin: i64 = (x - rad - half).ceil().as_();
    let ihmin: i64 = (y - rad - half).ceil().as_();
    let iwmax: i64 = (x + rad - half).floor().as_();
    let ihmax: i64 = (y + rad - half).floor().as_();
    let mut res = Vec::<usize>::new();
    for iw in iwmin..iwmax + 1 {
        if iw < 0 || iw >= width.try_into().unwrap() {
            continue;
        }
        for ih in ihmin..ihmax + 1 {
            if ih < 0 || ih >= height.try_into().unwrap() {
                continue;
            }
            let w: Real = iw.as_() + half; // pixel center
            let h: Real = ih.as_() + half; // pixel center
            if (w - x) * (w - x) + (h - y) * (h - y) > rad * rad {
                continue;
            }
            let idata = ih as usize * width + iw as usize;
            res.push(idata);
        }
    }
    res
}

/// * `transform` - 3x3 homogeneous transformation matrix with **column major** order
pub fn fill<Real, VAL>(
    img_data: &mut [VAL],
    width: usize,
    x: &[Real; 2],
    transform_world2pix: &[Real; 9],
    rad_pix: Real,
    color: VAL,
) where
    Real: num_traits::Float + 'static + AsPrimitive<i64> + nalgebra::RealField,
    i64: AsPrimitive<Real>,
    VAL: Copy,
{
    let height = img_data.len() / width;
    let a = del_geo_core::mat3_col_major::transform_homogeneous(transform_world2pix, x).unwrap();
    let pixs = pixels_in_point(a[0], a[1], rad_pix, width, height);
    for idata in pixs {
        img_data[idata] = color;
    }
}


#[allow(clippy::identity_op)]
pub fn stroke_dda<Real, VAL>(
    img_data: &mut [VAL],
    width: usize,
    x: &[Real; 2],
    rad: Real,
    transform: &[Real; 9],
    color: VAL,
) where
    Real: num_traits::Float + 'static + AsPrimitive<i64> + nalgebra::RealField + AsPrimitive<usize>,
    i64: AsPrimitive<Real>,
    usize: AsPrimitive<Real>,
    i32: AsPrimitive<Real>,
    VAL: Copy
{
    let num_theta = 32;
    let two = Real::one() + Real::one();
    let dtheta: Real = two * Real::pi() / num_theta.as_();
    for i_theta in 0..num_theta {
        let theta0 = dtheta * i_theta.as_();
        let theta1 = dtheta * ((i_theta + 1) % num_theta).as_();
        let p0 = [
            x[0] + rad * num_traits::Float::cos(theta0),
            x[1] + rad * num_traits::Float::sin(theta0)];
        let p1 = [
            x[0] + rad * num_traits::Float::cos(theta1),
            x[1] + rad * num_traits::Float::sin(theta1) ];
        let q0 = del_geo_core::mat3_col_major::transform_homogeneous(&transform, &p0).unwrap();
        let q1 = del_geo_core::mat3_col_major::transform_homogeneous(&transform, &p1).unwrap();
        crate::rasterize_line::draw_dda(
            img_data, width, &q0, &q1, color);
    }

}