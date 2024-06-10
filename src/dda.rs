use num_traits::AsPrimitive;

pub fn line_scr<Real>(
    img_data: &mut [u8],
    width: usize,
    p0: &[Real; 2],
    p1: &[Real; 2],
    i_color: u8,
) where
    Real: num_traits::Float + 'static + Copy + AsPrimitive<usize> + std::fmt::Debug,
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

pub fn line<Real>(
    img_data: &mut [u8],
    width: usize,
    p0: &[Real; 2],
    p1: &[Real; 2],
    transform: &nalgebra::Matrix3<Real>,
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
    let q0 = transform * nalgebra::Vector3::<Real>::new(p0[0], p0[1], Real::one());
    let q1 = transform * nalgebra::Vector3::<Real>::new(p1[0], p1[1], Real::one());
    line_scr(img_data, width, &[q0.x, q0.y], &[q1.x, q1.y], i_color);
}
