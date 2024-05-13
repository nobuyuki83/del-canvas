use num_traits::AsPrimitive;

pub fn pixels_in_line<Real>(
    x0: Real,
    y0: Real,
    x1: Real,
    y1: Real,
    rad: Real,
    width: usize,
    height: usize)
    -> Vec<usize>
where Real: num_traits::Float + 'static + AsPrimitive<i64>,
    i64: AsPrimitive<Real>
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
            std::cmp::max(ih0_max, ih1_max))
    };
    let sqlen = (x1-x0) * (x1-x0) + (y1-y0) * (y1-y0);
    let mut res = Vec::<usize>::new();
    for ih in ih_min..ih_max + 1 {
        if ih < 0 || ih >= height.try_into().unwrap() { continue; }
        for iw in iw_min..iw_max + 1 {
            if iw < 0 || iw >= width.try_into().unwrap() { continue; }
            let w: Real = iw.as_() + half; // pixel center
            let h: Real = ih.as_() + half; // pixel center
            let t = ((w-x0)*(x1-x0) + (h-y0)*(y1-y0))/sqlen;
            let sqdist = if t < Real::zero() {
                (w-x0)*(w-x0) + (h-y0)*(h-y0)
            } else if t > Real::one() {
                (w-x1)*(w-x1) + (h-y1)*(h-y1)
            } else {
                (w-x0)*(w-x0) + (h-y0)*(h-y0) - sqlen * t * t
            };
            if sqdist > rad * rad { continue; }
            let idata = ih as usize * width + iw as usize;
            res.push(idata);
        }
    }
    res
}


pub fn pixels_in_point<Real>(
    x: Real, y: Real, rad: Real,
    width: usize, height: usize) -> Vec<usize>
    where Real: num_traits::Float + 'static + AsPrimitive<i64>,
          i64: AsPrimitive<Real>,
{
    let half: Real = Real::one() / (Real::one() + Real::one());
    let iwmin: i64 = (x - rad - half).ceil().as_();
    let ihmin: i64 = (y - rad - half).ceil().as_();
    let iwmax: i64 = (x + rad - half).floor().as_();
    let ihmax: i64 = (y + rad - half).floor().as_();
    let mut res = Vec::<usize>::new();
    for iw in iwmin..iwmax + 1 {
        if iw < 0 || iw >= width.try_into().unwrap() { continue; }
        for ih in ihmin..ihmax + 1 {
            if ih < 0 || ih >= height.try_into().unwrap() { continue; }
            let w: Real = iw.as_() + half; // pixel center
            let h: Real = ih.as_() + half; // pixel center
            if (w - x) * (w - x) + (h - y) * (h - y) > rad * rad { continue; }
            let idata = ih as usize * width + iw as usize;
            res.push(idata);
        }
    }
    res
}

pub fn line<T, VAL>(
    img_data: &mut [VAL],
    width: usize,
    p0 :&[T;2],
    p1: &[T;2],
    transform: &nalgebra::Matrix3::<T>,
    rad: T,
    color: VAL)
    where T: num_traits::Float + nalgebra::RealField + num_traits::AsPrimitive<i64>,
          i64: AsPrimitive<T>,
          VAL: Copy
{
    let height = img_data.len() / width;
    let a0 = transform * nalgebra::Vector3::<T>::new(p0[0], p0[1], T::one());
    let a1 = transform * nalgebra::Vector3::<T>::new(p1[0], p1[1], T::one());
    let pixs = pixels_in_line(a0.x, a0.y, a1.x, a1.y, rad, width, height);
    for idata in pixs {
        img_data[idata] = color;
    }
}

#[allow(clippy::identity_op)]
pub fn polyloop<T, VAL>(
    img_data: &mut [VAL],
    width: usize,
    //
    vtx2xy: &[T],
    transform: &nalgebra::Matrix3::<T>,
    point_size: T,
    color: VAL)
    where T: num_traits::Float + nalgebra::RealField + num_traits::AsPrimitive<i64>,
          i64: AsPrimitive<T>,
          VAL: Copy
{
    let n = vtx2xy.len() / 2;
    for i in 0..n {
        let j = (i + 1) % n;
        line(
            img_data,
            width,
            &[vtx2xy[i * 2 + 0], vtx2xy[i * 2 + 1]],
            &[vtx2xy[j * 2 + 0], vtx2xy[j * 2 + 1]],
            transform,
            point_size,
            color);
    }
}

#[allow(clippy::identity_op)]
pub fn point<Real, VAL>(
    img_data: &mut [VAL],
    width: usize,
    x: [Real;2],
    transform: &nalgebra::Matrix3::<Real>,
    rad: Real,
    color: VAL)
    where Real: num_traits::Float + 'static + AsPrimitive<i64> + nalgebra::RealField,
          i64: AsPrimitive<Real>,
          VAL: Copy
{
    let height = img_data.len() / width;
    let a = transform * nalgebra::Vector3::<Real>::new(x[0], x[1], Real::one());
    let pixs = pixels_in_point(a.x, a.y, rad, width, height);
    for idata in pixs {
        img_data[idata] = color;
    }
}
