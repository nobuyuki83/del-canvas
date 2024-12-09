use num_traits::AsPrimitive;

pub fn draw_dda<Real, VAL>(
    img_data: &mut [VAL],
    width: usize,
    p0: &[Real; 2],
    p1: &[Real; 2],
    i_color: VAL,
) where
    Real: num_traits::Float + 'static + Copy + AsPrimitive<usize> + std::fmt::Debug,
    usize: AsPrimitive<Real>,
    VAL: Copy,
{
    let height = img_data.len() / width;
    let width_f: Real = width.as_();
    let height_f: Real = height.as_();
    let zero = Real::zero();
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
        if x >= zero && x < width_f && y >= zero && y < height_f {
            let ix: usize = x.as_();
            let iy: usize = y.as_();
            img_data[iy * width + ix] = i_color;
        }
        x = x + slope_x;
        y = y + slope_y;
    }
}

/// * `transform` - 3x3 homogeneous transformation matrix with **column major** order
pub fn draw_dda_with_transformation<Real, VAL>(
    img_data: &mut [VAL],
    width: usize,
    p0: &[Real; 2],
    p1: &[Real; 2],
    transform: &[Real; 9],
    i_color: VAL,
) where
    Real: num_traits::Float
        + std::fmt::Debug
        + std::ops::MulAssign
        + std::ops::AddAssign
        + 'static
        + Copy
        + AsPrimitive<usize>,
    usize: AsPrimitive<Real>,
    VAL: Copy,
{
    let q0 = del_geo_core::mat3_col_major::transform_homogeneous(transform, p0).unwrap();
    let q1 = del_geo_core::mat3_col_major::transform_homogeneous(transform, p1).unwrap();
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
    Real: num_traits::Float + 'static + AsPrimitive<usize>,
    usize: AsPrimitive<Real>,
{
    let half: Real = Real::one() / (Real::one() + Real::one());
    let aabbi = {
        let aabb = del_geo_core::aabb2::from_two_points(&[x0, y0], &[x1, y1], rad);
        del_geo_core::aabb2::rasterize(&aabb, &(width, height))
    };
    let sqlen = (x1 - x0) * (x1 - x0) + (y1 - y0) * (y1 - y0);
    let mut res = Vec::<usize>::new();
    for ih in aabbi[1]..aabbi[3] {
        for iw in aabbi[0]..aabbi[2] {
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
            res.push(ih * width + iw);
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
    transform_world2pix: &[T; 9],
    thickness: T,
    color: VAL,
) where
    T: num_traits::Float + nalgebra::RealField + num_traits::AsPrimitive<usize>,
    usize: AsPrimitive<T>,
    VAL: Copy,
{
    let height = img_data.len() / width;
    let a0 = del_geo_core::mat3_col_major::transform_homogeneous(transform_world2pix, p0).unwrap();
    let a1 = del_geo_core::mat3_col_major::transform_homogeneous(transform_world2pix, p1).unwrap();
    let pixs = pixels_in_line(a0[0], a0[1], a1[0], a1[1], thickness, width, height);
    for i_data in pixs {
        img_data[i_data] = color;
    }
}
