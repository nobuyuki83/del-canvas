use num_traits::AsPrimitive;

pub struct Camera3<T> {
    pub w: usize,
    pub h: usize,
    o: nalgebra::Vector3<T>,
    d: nalgebra::Vector3<T>,
    cx: nalgebra::Vector3<T>,
    cy: nalgebra::Vector3<T>,
}

impl<T> Camera3<T>
where
    T: nalgebra::RealField + 'static + Copy,
    f64: AsPrimitive<T>,
    usize: AsPrimitive<T>,
{
    pub fn new(w: usize, h: usize, o: nalgebra::Vector3<T>, d: nalgebra::Vector3<T>) -> Self {
        let d = d.normalize();
        let cx =
            nalgebra::Vector3::<T>::new(w.as_() * 0.5135.as_() / h.as_(), T::zero(), T::zero());
        let cy = cx.cross(&d).normalize() * 0.5135.as_();
        Camera3 { w, h, o, d, cx, cy }
    }

    pub fn ray(&self, x0: T, y0: T) -> (nalgebra::Vector3<T>, nalgebra::Vector3<T>) {
        let d = self.cx * (x0 / self.w.as_() - 0.5.as_())
            + self.cy * (y0 / self.h.as_() - 0.5.as_())
            + self.d;
        (self.o + d * 140.as_(), d.normalize())
    }
}
