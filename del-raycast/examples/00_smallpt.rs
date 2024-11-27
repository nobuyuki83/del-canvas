type Real = f64;
type Vector = nalgebra::Vector3<Real>;

struct Ray {
    o: Vector,
    d: Vector,
}

impl Ray {
    fn new(o: Vector, d: Vector) -> Self {
        Self { o, d }
    }
}

enum ReflT {
    DIFF,
    SPEC,
} // material types, used in radiance()

struct Sphere {
    rad: Real,
    // radius
    p: Vector,
    // position
    e: Vector,
    // emission
    c: Vector,
    // color
    refl: ReflT, // reflection type (DIFFuse, SPECular, REFRactive)
}

impl Sphere {
    fn new(rad_: Real, p_: Vector, e_: Vector, c_: Vector, refl_: ReflT) -> Self {
        Self {
            rad: rad_,
            p: p_,
            e: e_,
            c: c_,
            refl: refl_,
        }
    }

    fn intersect(&self, r: &Ray) -> Option<Real> {
        let Some(t) = del_geo_nalgebra::sphere::intersection_ray(&self.p, self.rad, &r.o, &r.d)
        else {
            return None;
        };
        Some(t)
    }
}

fn intersect(r: &Ray, spheres: &[Sphere]) -> Option<(Real, usize)> {
    const INF: Real = 1e20;
    let mut t_min = INF;
    let mut id = usize::MAX;
    for (isphere, sphere) in spheres.iter().enumerate() {
        let Some(t) = sphere.intersect(r) else {
            continue;
        };
        if t < t_min {
            t_min = t;
            id = isphere;
        }
    }
    if t_min < INF {
        return Some((t_min, id));
    }
    None
}

fn radiance(r: &Ray, depth: i64, spheres: &[Sphere], rng: &mut rand::rngs::ThreadRng) -> Vector {
    use rand::Rng;
    let Some((t, id)) = intersect(r, spheres) else {
        return Vector::new(0., 0., 0.); // if ray miss, return black
    };
    let obj = &spheres[id]; // the hit object
    let hit_pos = r.o + r.d * t; // hit pos
    let hit_nrm = (hit_pos - obj.p).normalize();
    let hit_nrm = if hit_nrm.dot(&r.d) < 0. {
        hit_nrm
    } else {
        hit_nrm * -1.
    };
    let mut f = obj.c;
    let depth = depth + 1;
    if depth > 5 {
        let p = if f.x > f.y && f.x > f.z {
            f.x
        } else if f.y > f.z {
            f.y
        } else {
            f.z
        }; // max refl
        if rng.gen::<Real>() < p {
            f = f * (1. / p);
        } else {
            return obj.e;
        } //R.R.
    }
    let hit_pos_offset = hit_pos + hit_nrm.scale(1.0e-3);
    let next_dir = match obj.refl {
        ReflT::DIFF => del_raycast::sampling::hemisphere_cos_weighted(
            &hit_nrm,
            &[rng.gen::<Real>(), rng.gen::<Real>()],
        ),
        ReflT::SPEC => r.d - hit_nrm * 2. * hit_nrm.dot(&r.d),
    };
    obj.e
        + f.component_mul(&radiance(
            &Ray::new(hit_pos_offset, next_dir),
            depth,
            spheres,
            rng,
        ))
}

fn main() {
    use crate::ReflT::{DIFF, SPEC};
    use rand::Rng;
    let spheres = [
        Sphere::new(
            1e5,
            Vector::new(1e5 + 1., 40.8, 81.6),
            Vector::new(0., 0., 0.),
            Vector::new(0.75, 0.25, 0.25),
            DIFF,
        ), //Left
        Sphere::new(
            1e5,
            Vector::new(-1e5 + 99., 40.8, 81.6),
            Vector::new(0., 0., 0.),
            Vector::new(0.25, 0.25, 0.75),
            DIFF,
        ), //Rght
        Sphere::new(
            1e5,
            Vector::new(50., 40.8, 1e5),
            Vector::new(0., 0., 0.),
            Vector::new(0.75, 0.75, 0.75),
            DIFF,
        ), //Back
        Sphere::new(
            1e5,
            Vector::new(50., 40.8, -1e5 + 170.),
            Vector::new(0., 0., 0.),
            Vector::new(0., 0., 0.),
            DIFF,
        ), //Frnt
        Sphere::new(
            1e5,
            Vector::new(50., 1e5, 81.6),
            Vector::new(0., 0., 0.),
            Vector::new(0.75, 0.75, 0.75),
            DIFF,
        ), //Botm
        Sphere::new(
            1e5,
            Vector::new(50., -1e5 + 81.6, 81.6),
            Vector::new(0., 0., 0.),
            Vector::new(0.75, 0.75, 0.75),
            DIFF,
        ), //Top
        Sphere::new(
            16.5,
            Vector::new(27., 16.5, 47.),
            Vector::new(0., 0., 0.),
            Vector::new(1., 1., 1.) * 0.999,
            SPEC,
        ), //Mirr
        Sphere::new(
            600.,
            Vector::new(50., 681.6 - 0.27, 81.6),
            Vector::new(12., 12., 12.),
            Vector::new(0., 0., 0.),
            DIFF,
        ), //Lite
    ];
    let mut rng = rand::thread_rng();
    let samps = 5;
    let cam = del_raycast::cam3::Camera3::new(
        1024,
        768,
        Vector::new(50., 52., 295.6),
        Vector::new(0., -0.042612, -1.),
    );
    let mut img = Vec::<image::Rgb<f32>>::new();
    img.resize(cam.w * cam.h, image::Rgb([0_f32; 3]));
    for y in 0..cam.h {
        // Loop over image rows
        for x in 0..cam.w {
            let mut c = Vector::new(0., 0., 0.);
            for sy in 0..2 {
                // 2x2 subpixel rows
                for sx in 0..2 {
                    // 2x2 subpixel cols
                    for _s in 0..samps {
                        let dx = del_raycast::sampling::tent(rng.gen::<Real>());
                        let dy = del_raycast::sampling::tent(rng.gen::<Real>());
                        let sx = sx as Real;
                        let x0 = (sx as Real + 0.5 + dx) / 2. + x as Real;
                        let y0 = (sy as Real + 0.5 + dy) / 2. + y as Real;
                        let ray = cam.ray(x0, y0);
                        let ray = Ray::new(ray.0, ray.1);
                        c = c + radiance(&ray, 0, &spheres, &mut rng);
                    } // Camera rays are pushed ^^^^^ forward to start in interior
                }
            }
            c *= 0.25 * (1. / samps as Real);
            img[(cam.h - 1 - y) * cam.w + x] = image::Rgb([c.x as f32, c.y as f32, c.z as f32]);
        }
    }
    {
        use image::codecs::hdr::HdrEncoder;
        let mut file = std::fs::File::create("target/00_smallpt.hdr").unwrap();
        let enc = HdrEncoder::new(file);
        let _ = enc.encode(&img, cam.w, cam.h);
    }
}
