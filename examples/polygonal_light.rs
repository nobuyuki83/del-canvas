use num_traits::float::FloatConst;
type Vector = nalgebra::Vector3<f64>;

struct Ray {
    o: Vector,
    d: Vector,
}

impl Ray {
    fn new(o: Vector, d: Vector) -> Self {
        Self { o, d }
    }
}

fn radiance(ray: &Ray, vtx2xyz: &[f64], _rng: &mut rand::rngs::ThreadRng) -> Vector {
    let floor = (Vector::new(0., 0., 0.), Vector::new(0., 1., 0.));
    let mut i_material = 0; // none
    let mut hit_depth = Option::<f64>::None;
    {
        let hd = del_geo::plane::intersection_ray3(&floor.0, &floor.1, &ray.o, &ray.d);
        if hd.is_some() {
            if hit_depth.is_none() || Some(hd) < Some(hit_depth) {
                hit_depth = hd;
                i_material = 1;
            }
        }
    }
    {
        let hd = del_msh::polyloop3::winding_number(
            vtx2xyz,
            ray.o.as_slice().try_into().unwrap(),
            ray.d.as_slice().try_into().unwrap(),
        );
        if (hd + 1.0).abs() < 1.0e-3 {
            let hd = Some(0.1);
            if hit_depth.is_none() || Some(hd) < Some(hit_depth) {
                hit_depth = hd;
                i_material = 2;
            }
        }
    }
    match i_material {
        1 => {
            let n = Vector::new(0., 1., 0.);
            let m = nalgebra::Matrix3::new(1.0, 0., 0., 0., 1.0, 0., 0., 0., 1.0);
            let hit_pos = ray.o + hit_depth.unwrap() * ray.d;
            let mut a = 0.;
            let num_vtx = vtx2xyz.len() / 3;
            for ino0 in 0..num_vtx {
                let ino1 = (ino0 + 1) % num_vtx;
                let v0 = Vector::from_row_slice(&vtx2xyz[ino0 * 3..(ino0 + 1) * 3]);
                let v1 = Vector::from_row_slice(&vtx2xyz[ino1 * 3..(ino1 + 1) * 3]);
                let v0 = (m * (v0 - hit_pos)).normalize();
                let v1 = (m * (v1 - hit_pos)).normalize();
                let t0 = v0.cross(&v1).normalize().dot(&n);
                let t1 = v0.dot(&v1).acos();
                a += t0 * t1;
            }
            a *= -f64::FRAC_1_PI();
            Vector::new(a, a, a)
        }
        2 => Vector::new(1., 1., 1.),
        _ => Vector::new(0., 0., 0.),
    }
}

fn main() {
    let vtx2xyz: Vec<f64> = vec![0., 30., 0., 30., 1., 0., 0., 60., 0., -30., 1.0, 0.];
    let mut rng = rand::thread_rng();
    let cam = del_canvas::cam3::Camera3::new(
        1024,
        768,
        Vector::new(50., 52., 295.6),
        Vector::new(0., -0.042612, -1.),
    );
    let mut img = Vec::<image::Rgb<f32>>::new();
    img.resize(cam.w * cam.h, image::Rgb([0_f32; 3]));
    for y in 0..cam.h {
        // Loop over image rows
        // dbg!(stderr, "\rRendering (%d spp) %5.2f%%", samps * 4, 100. * y / (h - 1));
        for x in 0..cam.w {
            let ray = cam.ray(0.5 + x as f64, 0.5 + y as f64);
            let ray = Ray::new(ray.0, ray.1);
            let c = radiance(&ray, &vtx2xyz, &mut rng);
            img[(cam.h - 1 - y) * cam.w + x] = image::Rgb([c.x as f32, c.y as f32, c.z as f32]);
        }
    }
    {
        use image::codecs::hdr::HdrEncoder;
        let file = std::fs::File::create("target/02_img.hdr").unwrap();
        let enc = HdrEncoder::new(file);
        let _ = enc.encode(&img, cam.w, cam.h);
    }
}
