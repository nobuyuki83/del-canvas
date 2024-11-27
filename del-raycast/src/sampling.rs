use num_traits::AsPrimitive;

pub fn hemisphere_zup_cos_weighted<T>(r2: &[T; 2]) -> [T; 3]
where
    T: num_traits::Float + 'static + Copy,
    f64: AsPrimitive<T>,
{
    let z = (r2[0]).sqrt();
    let r = (T::one() - z * z).sqrt();
    let phi: T = 2_f64.as_() * std::f64::consts::PI.as_() * r2[1];
    [r * phi.cos(), r * phi.sin(), z]
}

pub fn hemisphere_cos_weighted<T>(n: &nalgebra::Vector3<T>, r2: &[T; 2]) -> nalgebra::Vector3<T>
where
    T: num_traits::Float + Copy + 'static + nalgebra::RealField,
    f64: AsPrimitive<T>,
{
    let h0 = hemisphere_zup_cos_weighted(r2);
    let t = if num_traits::Float::abs(n[0]) > 0.1.as_() {
        nalgebra::Vector3::<T>::new(T::zero(), T::one(), T::zero())
    } else {
        nalgebra::Vector3::<T>::new(T::one(), T::zero(), T::zero())
    };
    let u = t.cross(n).normalize(); // orthogonal to w
    let v = n.cross(&u); // orthogonal to w and u
    (u.scale(h0[0]) + v.scale(h0[1]) + n.scale(h0[2])).normalize()
}

pub fn tent<T>(r0: T) -> T
where
    T: num_traits::Float + 'static + Copy,
    f64: AsPrimitive<T>,
{
    let r1: T = 2_f64.as_() * r0;
    if r1 < T::one() {
        r1.sqrt() - T::one()
    } else {
        let tmp: T = 2_f64.as_() - r1;
        T::one() - tmp.sqrt()
    } // tent filter (-1 .. +1 )
}
