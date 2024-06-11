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
#[allow(clippy::identity_op)]
pub fn fill<Real, VAL>(
    img_data: &mut [VAL],
    width: usize,
    x: &[Real; 2],
    transform: &[Real; 9],
    rad: Real,
    color: VAL,
) where
    Real: num_traits::Float + 'static + AsPrimitive<i64> + nalgebra::RealField,
    i64: AsPrimitive<Real>,
    VAL: Copy,
{
    let height = img_data.len() / width;
    let a = del_geo::mat3::transform_homogeneous(transform, x).unwrap();
    let pixs = pixels_in_point(a[0], a[1], rad, width, height);
    for idata in pixs {
        img_data[idata] = color;
    }
}
