use num_traits::AsPrimitive;

pub fn u8rgb_from_i32(color: i32) -> (u8, u8, u8) {
    let r = ((color & 0xff0000) >> 16) as u8;
    let g = ((color & 0x00ff00) >> 8) as u8;
    let b = (color & 0x0000ff) as u8;
    (r, g, b)
}

pub fn i32_form_u8rgb(r: u8, g: u8, b: u8) -> i32 {
    let r = (r as i32) * 0x010000;
    let g = (g as i32) * 0x000100;
    let b = b as i32;
    r + g + b
}

pub fn i32_from_f32rgb(r: f32, g: f32, b: f32) -> i32 {
    let r = (r * 255.) as i32;
    let g = (g * 255.) as i32;
    let b = (b * 255.) as i32;
    assert!((0..=255).contains(&r));
    assert!((0..=255).contains(&g));
    assert!((0..=255).contains(&b));
    r * 0x010000 + g * 0x000100 + b
}

pub fn rgb_from_hsv<T>(h: T, s: T, v: T) -> (T, T, T)
where
    T: num_traits::Float + std::ops::MulAssign + AsPrimitive<i32>,
    i32: AsPrimitive<T>,
{
    assert!(h >= T::zero() && h <= T::one());
    if s < T::zero() {
        return (v, v, v);
    }
    //
    let one = T::one();
    let six = one + one + one + one + one + one;
    let (mut r, mut g, mut b) = (v, v, v);
    let h = h * six;
    let i: i32 = h.as_();
    let f: T = h - i.as_();
    match i {
        0 => {
            g *= one - s * (one - f);
            b *= one - s;
        }
        1 => {
            r *= one - s * f;
            b *= one - s;
        }
        2 => {
            r *= one - s;
            b *= one - s * (one - f);
        }
        3 => {
            r *= one - s;
            g *= one - s * f;
        }
        4 => {
            r *= one - s * (one - f);
            g *= one - s;
        }
        5 => {
            g *= one - s;
            b *= one - s * f;
        }
        _ => panic!(),
    }
    (r, g, b)
}
