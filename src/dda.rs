use num_traits::AsPrimitive;

pub fn line<Real>(
    x0: Real,
    y0: Real,
    x1: Real,
    y1: Real,
    img_data: &mut [u8],
    width: usize)
where Real : num_traits::Float + 'static + Copy + AsPrimitive<usize> + std::fmt::Debug
{
    let dx = x1 - x0;
    let dy = y1 - y0;
    let step = if dx.abs() > dy.abs() { dx.abs() } else { dy.abs() };
    let slope_y = dy / step;
    let slope_x = dx / step;
    let mut x = x0;
    let mut y = y0;
    while (x - x0).abs() <= (x1 - x0).abs() && (y - y0).abs() <= (y1 - y0).abs() {
        let ix: usize = x.as_();
        let iy: usize = y.as_();
        img_data[iy * width + ix] = 0;
        x = x + slope_x;
        y = y + slope_y;
    }
}