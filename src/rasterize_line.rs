use num_traits::AsPrimitive;

pub fn draw_dda<Real, VAL>(
    img_data: &mut [VAL],
    width: usize,
    p0: &[Real; 2],
    p1: &[Real; 2],
    i_color: VAL,
) where
    Real: num_traits::Float + 'static + Copy + AsPrimitive<usize> + std::fmt::Debug,
    VAL: Copy,
{
    let dx = p1[0] - p0[0];
    let dy = p1[1] - p0[1];
    let step = if dx.abs() > dy.abs() {
        dx.abs()
    } else {
        dy.abs()
    };
    let slope_y = dy / step;
    let slope_x = dx / step;
    let mut x = p0[0];
    let mut y = p0[1];
    while (x - p0[0]).abs() <= (p1[0] - p0[0]).abs() && (y - p0[1]).abs() <= (p1[1] - p0[1]).abs() {
        let ix: usize = x.as_();
        let iy: usize = y.as_();
        img_data[iy * width + ix] = i_color;
        x = x + slope_x;
        y = y + slope_y;
    }
}

/// * `transform` - 3x3 homogeneous transformation matrix with **column major** order
pub fn draw_dda_with_transformation<Real>(
    img_data: &mut [u8],
    width: usize,
    p0: &[Real; 2],
    p1: &[Real; 2],
    transform: &[Real; 9],
    i_color: u8,
) where
    Real: num_traits::Float
        + std::fmt::Debug
        + std::ops::MulAssign
        + std::ops::AddAssign
        + 'static
        + Copy
        + AsPrimitive<usize>,
{
    let q0 = del_geo::mat3::transform_homogeneous(transform, p0).unwrap();
    let q1 = del_geo::mat3::transform_homogeneous(transform, p1).unwrap();
    draw_dda(img_data, width, &q0, &q1, i_color);
}

pub fn pixels_in_line<Real>(
    x0: Real,
    y0: Real,
    x1: Real,
    y1: Real,
    rad: Real,
    width: usize,
    height: usize,
) -> Vec<usize>
where
    Real: num_traits::Float + 'static + AsPrimitive<i64>,
    i64: AsPrimitive<Real>,
{
    let half: Real = Real::one() / (Real::one() + Real::one());
    let (iw_min, iw_max, ih_min, ih_max) = {
        let iw0_min: i64 = (x0 - rad - half).ceil().as_();
        let ih0_min: i64 = (y0 - rad - half).ceil().as_();
        let iw1_min: i64 = (x1 - rad - half).ceil().as_();
        let ih1_min: i64 = (y1 - rad - half).ceil().as_();
        let iw0_max: i64 = (x0 + rad - half).floor().as_();
        let ih0_max: i64 = (y0 + rad - half).floor().as_();
        let iw1_max: i64 = (x1 + rad - half).floor().as_();
        let ih1_max: i64 = (y1 + rad - half).floor().as_();
        (
            std::cmp::min(iw0_min, iw1_min),
            std::cmp::max(iw0_max, iw1_max),
            std::cmp::min(ih0_min, ih1_min),
            std::cmp::max(ih0_max, ih1_max),
        )
    };
    let sqlen = (x1 - x0) * (x1 - x0) + (y1 - y0) * (y1 - y0);
    let mut res = Vec::<usize>::new();
    for ih in ih_min..ih_max + 1 {
        if ih < 0 || ih >= height.try_into().unwrap() {
            continue;
        }
        for iw in iw_min..iw_max + 1 {
            if iw < 0 || iw >= width.try_into().unwrap() {
                continue;
            }
            let w: Real = iw.as_() + half; // pixel center
            let h: Real = ih.as_() + half; // pixel center
            let t = ((w - x0) * (x1 - x0) + (h - y0) * (y1 - y0)) / sqlen;
            let sqdist = if t < Real::zero() {
                (w - x0) * (w - x0) + (h - y0) * (h - y0)
            } else if t > Real::one() {
                (w - x1) * (w - x1) + (h - y1) * (h - y1)
            } else {
                (w - x0) * (w - x0) + (h - y0) * (h - y0) - sqlen * t * t
            };
            if sqdist > rad * rad {
                continue;
            }
            let idata = ih as usize * width + iw as usize;
            res.push(idata);
        }
    }
    res
}

/// * `transform` - 3x3 homogeneous transformation matrix with **column major** order
pub fn draw_pixcenter<T, VAL>(
    img_data: &mut [VAL],
    width: usize,
    p0: &[T; 2],
    p1: &[T; 2],
    transform: &[T;9],
    rad: T,
    color: VAL,
) where
    T: num_traits::Float + nalgebra::RealField + num_traits::AsPrimitive<i64>,
    i64: AsPrimitive<T>,
    VAL: Copy,
{
    let height = img_data.len() / width;
    let a0 = del_geo::mat3::transform_homogeneous(transform, p0).unwrap();
    let a1 = del_geo::mat3::transform_homogeneous(transform, p1).unwrap();
    let pixs = pixels_in_line(a0[0], a0[1], a1[0], a1[1], rad, width, height);
    for idata in pixs {
        img_data[idata] = color;
    }
}
