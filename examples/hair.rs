use num_traits::Pow;

fn gauss(x: f32, s: f32) -> f32 {
    (-0.5f32 * x * x / (s * s)).exp() / (s * (std::f32::consts::PI * 2f32).sqrt())
}

fn main() {
    type Vec3 = nalgebra::Vector3<f32>;
    let (width, height) = (512usize, 512usize);
    let mut img_data = vec![image::Rgb::<f32>::from([0.8, 0.8, 0.8]); height * width * 3];
    let transform_pix2glb = nalgebra::Matrix3::<f32>::new(
        2f32 / (width as f32),
        0f32,
        -1f32,
        0f32,
        -2f32 / (height as f32),
        1f32,
        0f32,
        0f32,
        1f32,
    );
    let light_pos = Vec3::new(1.0, 1.0, 1.0);
    let alpha_rad = (-10f32) * std::f32::consts::PI / 180f32;
    let beta_rad = (10f32) * std::f32::consts::PI / 180f32;
    let eta = 1.55f32;
    let fresnel_0 = ((1f32 - eta) / (1f32 + eta)).pow(2);
    for i_w in 0..width {
        for i_h in 0..height {
            let org = Vec3::new(i_w as f32 + 0.5f32, i_h as f32 + 0.5f32, 1f32);
            let org = transform_pix2glb * org;
            let dir = Vec3::new(0f32, 0f32, -1f32);
            let Some((hit_pos, hit_normal, _depth)) = del_geo_nalgebra::sphere::intersection_ray(
                &nalgebra::Vector3::new(0., 0., 0.),
                0.5,
                &org,
                &dir,
            ) else {
                continue;
            };
            assert!((hit_normal.norm() - 1f32).abs() < 1.0e-5);
            // pointing in the direction from the root toward the tip
            let fiber_dir = Vec3::new(0., -1., 0.);
            let fiber_dir = (fiber_dir - fiber_dir.dot(&hit_normal) * hit_normal).normalize();
            let binormal = fiber_dir.cross(&hit_normal);
            let w_i = (light_pos - hit_pos).normalize();
            let w_o = Vec3::new(0., 0., 1.);
            let theta_i = fiber_dir.dot(&w_i).atan2(hit_normal.dot(&w_i));
            let theta_o = fiber_dir.dot(&w_o).atan2(hit_normal.dot(&w_o));
            let theta_h = (theta_i + theta_o) * 0.5f32;
            let m_r = gauss(theta_h - alpha_rad, beta_rad);
            let _m_tt = gauss(theta_h + alpha_rad * 0.5f32, beta_rad * 0.5f32);
            let _m_trt = gauss(theta_h + alpha_rad * 1.5f32, beta_rad * 2f32);
            let phi_i = binormal.dot(&w_i).atan2(hit_normal.dot(&w_i));
            let phi_o = binormal.dot(&w_o).atan2(hit_normal.dot(&w_o));
            let phi = phi_i - phi_o;
            let n_r = {
                let x = (0.5f32 + 0.5f32 * w_i.dot(&w_o)).sqrt(); // cos(tdheta/2)
                                                                  // reflection ratio by Schlick's approximation
                let a = fresnel_0 + (1f32 - fresnel_0) * (1f32 - x).pow(5);
                0.25f32 * (phi * 0.5f32).cos() * a
            };
            // dbg!(n_r);
            /*
            let n_trt = {
                let T = pow(hair_color.rgb,vec3(0.8/cosThetaD));
                let D = (17*cosPhiD-16.78).exp();
                let f = schlickFresnel(cosThetaD*0.5);
                let A = (1f32-f).pow(2) * f * T * T;
                0.5 * A * D
            };
             */
            let s_r = 10f32 * (m_r * n_r);

            // dbg!(theta_h, m_r);
            img_data[i_h * width + i_w] = image::Rgb::<f32>::from([s_r, s_r, s_r]);
        }
    }
    let file = std::fs::File::create("target/hair.hdr").unwrap();
    let w = std::io::BufWriter::new(file);
    let enc = image::codecs::hdr::HdrEncoder::new(w);
    let _ = enc.encode(&img_data, width, height);
}
