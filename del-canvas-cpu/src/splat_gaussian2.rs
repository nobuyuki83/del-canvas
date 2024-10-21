pub fn tile2point<F>(
    point2aabbdepth: F,
    img_shape: (usize, usize),
    num_point: usize,
) -> (Vec<usize>, Vec<usize>, Vec<usize>)
where
    F: Fn(usize) -> ([f32; 4], f32),
{
    let mut idx2tilegauss: Vec<(u32, f32)> = vec![];
    let mut idx2point: Vec<usize> = vec![];
    const TILE_SIZE: usize = 16;
    let tile_shape: (usize, usize) = (img_shape.0 / TILE_SIZE, img_shape.1 / TILE_SIZE);
    for i_point in 0..num_point {
        let (aabb, depth) = point2aabbdepth(i_point);
        let ix0 = (aabb[0] / TILE_SIZE as f32).floor() as i32;
        let iy0 = (aabb[1] / TILE_SIZE as f32).floor() as i32;
        let ix1 = (aabb[2] / TILE_SIZE as f32).floor() as i32 + 1;
        let iy1 = (aabb[3] / TILE_SIZE as f32).floor() as i32 + 1;
        let mut tiles = std::collections::BTreeSet::<usize>::new();
        for ix in ix0..ix1 {
            assert_ne!(ix, ix1);
            if ix < 0 || ix >= (tile_shape.0 as i32) {
                continue;
            }
            let ix = ix as usize;
            for iy in iy0..iy1 {
                assert_ne!(iy, iy1);
                if iy < 0 || iy >= (tile_shape.1 as i32) {
                    continue;
                }
                let iy = iy as usize;
                let i_tile = iy * tile_shape.0 + ix;
                tiles.insert(i_tile);
            }
        }
        for i_tile in tiles {
            idx2tilegauss.push((i_tile as u32, depth));
            idx2point.push(i_point);
        }
    }
    let num_idx = idx2tilegauss.len();
    let mut jdx2idx: Vec<usize> = (0..num_idx).collect();
    jdx2idx.sort_by(|&idx0, &idx1| {
        let itile0 = idx2tilegauss[idx0].0;
        let itile1 = idx2tilegauss[idx1].0;
        if itile0 != itile1 {
            itile0.cmp(&itile1)
        } else {
            let depth0 = idx2tilegauss[idx0].1;
            let depth1 = idx2tilegauss[idx1].1;
            depth0.partial_cmp(&depth1).unwrap()
        }
    });
    let num_tile = tile_shape.0 * tile_shape.1;
    let mut tile2jdx = vec![0usize; num_tile + 1];
    {
        for jdx in 0..jdx2idx.len() {
            let idx0 = jdx2idx[jdx];
            let i_tile = idx2tilegauss[idx0].0 as usize;
            tile2jdx[i_tile + 1] += 1;
        }
        for i_tile in 0..num_tile {
            tile2jdx[i_tile + 1] += tile2jdx[i_tile];
        }
    }
    /*
    for i_tile in 0..num_tile {
        for &idx in &jdx2idx[tile2jdx[i_tile]..tile2jdx[i_tile+1]] {
            let i_point0 = idx2point[idx];
            println!("{} {} {} {}", i_tile, idx2tilegauss[idx].0, idx2tilegauss[idx].1, point2splat[i_point0*NDOF_SPLAT+9]);
        }
    }
     */
    /*
    for jdx in 0..jdx2idx.len() {
        let idx0 = jdx2idx[jdx];
        let i_point0 = idx2point[idx0];
        println!("{} {} {} {}", jdx, idx2tilegauss[idx0].0, idx2tilegauss[idx0].1, point2splat[i_point0*NDOF_SPLAT+9]);
    }
     */
    (tile2jdx, jdx2idx, idx2point)
}

struct Splat<'a> {
    data: &'a [f32; 10],
}

impl<'a> Splat<'a> {
    fn new(data: &[f32; 10]) -> Splat {
        Splat { data }
    }

    fn pos_pix(&self) -> nalgebra::Vector2<f32> {
        nalgebra::Vector2::<f32>::from_column_slice(&self.data[0..2])
    }

    fn sigma_inv(&self) -> &'a [f32; 3] {
        arrayref::array_ref!(self.data, 2, 3)
        // nalgebra::Matrix2::<f32>::new(abc[0], abc[1], abc[1], abc[2])
    }

    fn is_include_point(&self, p: &nalgebra::Vector2<f32>) -> bool {
        let aabb = arrayref::array_ref![self.data, 5, 4];
        del_geo_core::aabb2::is_inlcude_point(aabb, &[p[0], p[1]])
    }

    fn alpha(&self, t: &nalgebra::Vector2<f32>) -> f32 {
        let pos_pix = self.pos_pix();
        let t0 = t - pos_pix;
        let t0 = [t0[0], t0[1]];
        let e = del_geo_core::mat2_sym::mult_vec_from_both_sides(self.sigma_inv(), &t0, &t0);
        assert!(e >= 0f32);
        (-0.5 * e).exp()
    }
}

pub struct Gauss<'a> {
    // xyz, rgba, s0,s1,s2, q0,q1,q2,q3
    pub data: &'a [f32; 14],
}

impl<'a> Gauss<'a> {
    pub fn new(data: &[f32; 14]) -> Gauss {
        Gauss { data }
    }

    fn color(&self) -> nalgebra::Vector3<f32> {
        nalgebra::Vector3::new(self.data[3], self.data[4], self.data[5])
    }

    pub fn pos_world(&self) -> &[f32; 3] {
        arrayref::array_ref![self.data, 0, 3]
    }

    fn wdw_sigma2inv(&self, world2pix: &[f32; 6]) -> ([f32; 3], [[f32; 6]; 3]) {
        let dia = arrayref::array_ref![&self.data, 7, 3];
        let quat = arrayref::array_ref![&self.data, 10, 4];
        let (abc, dabcdt) = del_geo_core::mat2_sym::wdw_projected_spd_mat3(world2pix, quat, dia);
        let xyz = del_geo_core::mat2_sym::safe_inverse_preserve_positive_definiteness::<f32>(
            &abc, 1.0e-5f32,
        );
        let dxyz = del_geo_core::mat2_sym::wdw_inverse(&dabcdt, &xyz);
        (xyz, dxyz)
    }

    fn diagonal(&self) -> &'a [f32; 3] {
        //let dia = arrayref::array_ref![&self.data, 7, 3];
        //[dia[0], dia[1], dia[2]]
        arrayref::array_ref![self.data, 7, 3]
    }

    fn quaternion(&self) -> &'a [f32; 4] {
        arrayref::array_ref![self.data, 10, 4]
    }

    fn world2pix(&self, mvp: &[f32; 16], img_shape: &(usize, usize)) -> [f32; 6] {
        let mvp_grad = {
            let pos_world = self.pos_world();
            let pos_world = nalgebra::Vector3::<f32>::from_column_slice(pos_world);
            let mvp = nalgebra::Matrix4::<f32>::from_column_slice(mvp);
            del_geo_nalgebra::mat4::jacobian_transform(&mvp, &pos_world)
        };
        let ndc2pix = nalgebra::Matrix2x3::<f32>::new(
            0.5 * (img_shape.0 as f32),
            0.,
            0.,
            0.,
            -0.5 * (img_shape.1 as f32),
            0.,
        );
        let world2pix = ndc2pix * mvp_grad;
        *arrayref::array_ref![world2pix.as_slice(), 0, 6]
    }

    pub fn sigma2inv(&self, mvp: &[f32; 16], img_shape: &(usize, usize)) -> [f32; 3] {
        let world2pix = self.world2pix(mvp, img_shape);
        let w0 = {
            let world2pix = arrayref::array_ref![world2pix.as_slice(), 0, 6];
            let dia = self.diagonal();
            let quat = self.quaternion();
            let (abc, _dabcdt) =
                del_geo_core::mat2_sym::wdw_projected_spd_mat3(world2pix, quat, dia);
            del_geo_core::mat2_sym::safe_inverse_preserve_positive_definiteness::<f32>(
                &abc, 1.0e-5f32,
            )
        };
        w0
    }

    fn dw_sigma2inv(&self, mvp: &[f32; 16], img_shape: &(usize, usize)) -> [[f32; 6]; 3] {
        /*
        {
            let pos_world = gauss.pos_world();
            let pos_world = nalgebra::Vector3::<f32>::from_column_slice(pos_world);
            let mvp_grad = del_geo_nalgebra::mat4::jacobian_transform(&mvp, &pos_world);
            let ndc2pix = nalgebra::Matrix2x3::<f32>::new(
                0.5 * (img_shape.0 as f32),
                0.,
                0.,
                0.,
                -0.5 * (img_shape.1 as f32),
                0.,
            );
            ndc2pix * mvp_grad
        };
         */
        let world2pix = self.world2pix(mvp, img_shape);
        let (_sigma2inv, d_sigma2inv) = self.wdw_sigma2inv(&world2pix);
        d_sigma2inv
    }
}

/// how much the color of `i_splats` is visible
fn transfer_splats(splats: &[(usize, f32, nalgebra::Vector3<f32>)], i_splat: usize) -> f32 {
    let mut alpha = 1f32;
    for j_splat in 0..i_splat {
        alpha *= 1f32 - splats[j_splat].1;
    }
    alpha *= splats[i_splat].1;
    alpha
}

/// how transfer of `j_splat` affected by `i_splat`
fn dw_transfer_splats(
    splats: &[(usize, f32, nalgebra::Vector3<f32>)],
    i_splat: usize,
    j_splat: usize,
) -> f32 {
    if j_splat > i_splat {
        0f32
    } else if j_splat == i_splat {
        let mut didj = 1f32;
        for k_splat in 0..i_splat {
            didj *= 1f32 - splats[k_splat].1;
        }
        didj
    } else {
        let mut didj = 1f32;
        for k_splat in 0..i_splat + 1 {
            if k_splat != i_splat && k_splat != j_splat {
                didj *= 1f32 - splats[k_splat].1;
            } else if k_splat == j_splat {
                didj *= -1f32;
            } else if k_splat == i_splat {
                didj *= splats[k_splat].1;
            }
        }
        didj
    }
}

#[test]
fn test_transfer() {
    let splats0 = vec![
        (0usize, 0.4, nalgebra::Vector3::<f32>::zeros()),
        (0usize, 0.7, nalgebra::Vector3::<f32>::zeros()),
        (0usize, 0.5, nalgebra::Vector3::<f32>::zeros()),
        (0usize, 0.3, nalgebra::Vector3::<f32>::zeros()),
    ];
    let num_splats = splats0.len();
    let eps = 1.0e-4;
    for i_splat in 0..num_splats {
        let t0 = transfer_splats(&splats0, i_splat);
        for j_splat in 0..num_splats {
            let mut splats1 = splats0.clone();
            splats1[j_splat].1 += eps;
            let t1 = transfer_splats(&splats1, i_splat);
            let dt_diff = (t1 - t0) / eps;
            let dt_ana = dw_transfer_splats(&splats0, i_splat, j_splat);
            assert!((dt_diff - dt_ana).abs() < 2.0e-4);
        }
    }
}

pub fn rasterize(
    point2gauss: &[f32],
    point2splat: &[f32],
    tile2idx: &[usize],
    idx2point: &[usize],
    img_shape: (usize, usize),
    tile_size: usize,
) -> Vec<f32> {
    const NDOF_GAUSS: usize = 14; // xyz, rgba, s0,s1,s2, q0,q1,q2,q3
    const NDOF_SPLAT: usize = 10; // pos_pix(2) + abc(3) + aabb(4) + ndc_z(1)
    assert_eq!(point2gauss.len() % NDOF_GAUSS, 0);
    assert_eq!(point2splat.len() % NDOF_SPLAT, 0);
    assert_eq!(
        point2gauss.len() / NDOF_GAUSS,
        point2splat.len() / NDOF_SPLAT
    );
    let mut img_data = vec![0f32; img_shape.1 * img_shape.0 * 3];
    let tile_shape: (usize, usize) = (img_shape.0 / tile_size, img_shape.1 / tile_size);
    assert_eq!(tile_shape.0*tile_shape.1+1, tile2idx.len());
    for (ih, iw) in itertools::iproduct!(0..img_shape.1, 0..img_shape.0) {
        let i_tile = (ih / tile_size) * tile_shape.0 + (iw / tile_size);
        let pos_pix = nalgebra::Vector2::<f32>::new(iw as f32 + 0.5, ih as f32 + 0.5);
        let mut transfer = 1f32;
        for &i_point in &idx2point[tile2idx[i_tile]..tile2idx[i_tile + 1]] {
            let splat = Splat::new(arrayref::array_ref![
                point2splat,
                i_point * NDOF_SPLAT,
                NDOF_SPLAT
            ]);
            let gauss = Gauss::new(arrayref::array_ref![
                point2gauss,
                i_point * NDOF_GAUSS,
                NDOF_GAUSS
            ]);
            if !splat.is_include_point(&pos_pix) {
                continue;
            }
            let alpha = splat.alpha(&pos_pix);
            if alpha < 1.0e-3 {
                continue;
            }
            let color = gauss.color();
            img_data[(ih * img_shape.0 + iw) * 3] += color[0] * (transfer * alpha);
            img_data[(ih * img_shape.0 + iw) * 3 + 1] += color[1] * (transfer * alpha);
            img_data[(ih * img_shape.0 + iw) * 3 + 2] += color[2] * (transfer * alpha);
            transfer *= 1f32 - alpha;
            if transfer < 0.01 {
                break;
            }
        }
    }
    img_data
}

pub fn diff_point2gauss(
    point2gauss: &[f32],
    point2splat: &[f32],
    tile2idx: &[usize],
    idx2point: &[usize],
    img_shape: (usize, usize),
    tile_size: usize,
    mvp: &[f32; 16],
    dw_pix2rgb: &[f32],
) -> Vec<f32> {
    const NDOF_GAUSS: usize = 14; // xyz, rgba, s0,s1,s2, q0,q1,q2,q3
    const NDOF_SPLAT: usize = 10; // pos_pix(2) + abc(3) + aabb(4) + ndc_z(1)
    assert_eq!(point2gauss.len() % NDOF_GAUSS, 0);
    assert_eq!(point2splat.len() % NDOF_SPLAT, 0);
    assert_eq!(
        point2gauss.len() / NDOF_GAUSS,
        point2splat.len() / NDOF_SPLAT
    );
    assert_eq!(dw_pix2rgb.len(), img_shape.1 * img_shape.0 * 3);
    let mut dw_point2gauss = vec![0f32; point2gauss.len()];
    for (ih, iw) in itertools::iproduct!(0..img_shape.1, 0..img_shape.0) {
        let dw_rgb = arrayref::array_ref!(dw_pix2rgb, (ih * img_shape.0 + iw) * 3, 3);
        let tile_shape: (usize, usize) = (img_shape.0 / tile_size, img_shape.1 / tile_size);
        let i_tile = (ih / tile_size) * tile_shape.0 + (iw / tile_size);
        let pix_center = nalgebra::Vector2::<f32>::new(iw as f32 + 0.5, ih as f32 + 0.5);
        let splats = {
            let mut splats = Vec::<(usize, f32, nalgebra::Vector3<f32>)>::with_capacity(8);
            let mut transfer = 1f32;
            for &i_point in &idx2point[tile2idx[i_tile]..tile2idx[i_tile + 1]] {
                let splat = Splat::new(arrayref::array_ref![
                    point2splat,
                    i_point * NDOF_SPLAT,
                    NDOF_SPLAT
                ]);
                let gauss = Gauss::new(arrayref::array_ref![
                    point2gauss,
                    i_point * NDOF_GAUSS,
                    NDOF_GAUSS
                ]);
                if !splat.is_include_point(&pix_center) {
                    continue;
                }
                let alpha = splat.alpha(&pix_center);
                if alpha < 1.0e-3 {
                    continue;
                }
                let color = gauss.color();
                splats.push((i_point, alpha, color));
                transfer *= 1f32 - alpha;
                if transfer < 0.01 {
                    break;
                }
            }
            splats
        };
        for j_splat in 0..splats.len() {
            // compute derivative w.r.t parameters of j_splat
            let j_point = splats[j_splat].0; // point index
            {
                // color derivative
                let alpha = transfer_splats(&splats, j_splat);
                dw_point2gauss[j_point * NDOF_GAUSS + 3] += dw_rgb[0] * alpha;
                dw_point2gauss[j_point * NDOF_GAUSS + 4] += dw_rgb[1] * alpha;
                dw_point2gauss[j_point * NDOF_GAUSS + 5] += dw_rgb[2] * alpha;
            }
            let dcdaj = {
                let mut dcdaj = nalgebra::Vector3::<f32>::zeros();
                for i_splat in 0..splats.len() {
                    dcdaj += splats[i_splat].2 * dw_transfer_splats(&splats, i_splat, j_splat);
                }
                dcdaj[0] * dw_rgb[0] + dcdaj[1] * dw_rgb[1] + dcdaj[2] * dw_rgb[2]
            };
            let splat = Splat::new(arrayref::array_ref![
                point2splat,
                j_point * NDOF_SPLAT,
                NDOF_SPLAT
            ]);
            let gauss = Gauss::new(arrayref::array_ref![
                point2gauss,
                j_point * NDOF_GAUSS,
                NDOF_GAUSS
            ]);
            let d_sigma2inv = gauss.dw_sigma2inv(mvp, &img_shape);
            let d = pix_center - splat.pos_pix();
            let alpha = splat.alpha(&pix_center);
            assert!(alpha > 1.0e-3);
            for is in 0..6 {
                let v0 = dcdaj * alpha * -0.5 * d[0] * d[0] * d_sigma2inv[0][is];
                let v1 = dcdaj * alpha * -0.5 * d[0] * d[1] * d_sigma2inv[1][is] * 2.0;
                let v2 = dcdaj * alpha * -0.5 * d[1] * d[1] * d_sigma2inv[2][is];
                dw_point2gauss[j_point * NDOF_GAUSS + 7 + is] += v0 + v1 + v2;
            }
        }
    }
    dw_point2gauss
}

pub trait Splat2 {
    fn ndc_z(&self) -> f32;
    fn aabb(&self) -> &[f32; 4];
    fn property(&self) -> (&[f32; 2], &[f32; 3], &[f32; 3], f32);
}

/// rasterize 2D Gaussian splats without any acceleration for debugging
pub fn rasterize_naive<S, Path>(
    pnt2splat2: &[S],
    img_shape: (usize, usize),
    path: Path,
) -> anyhow::Result<()>
where
    S: Splat2,
    Path: AsRef<std::path::Path>,
{
    let idx2pnt = {
        let num_pnt = pnt2splat2.len();
        let mut idx2pnt: Vec<usize> = (0..num_pnt).collect();
        idx2pnt.sort_by(|&i, &j| {
            let zi = pnt2splat2[i].ndc_z() + 1f32;
            let zj = pnt2splat2[j].ndc_z() + 1f32;
            zi.partial_cmp(&zj).unwrap()
        });
        // idx2pnt.iter().enumerate().for_each(|(idx, &i_pnt)| println!("{} {}", idx, pnt2gs2[i_pnt].ndc_z));
        idx2pnt
    };
    // visualize as Gaussian without tile acceleration
    let mut img_data = vec![0f32; img_shape.1 * img_shape.0 * 3];
    for (ih, iw) in itertools::iproduct!(0..img_shape.1, 0..img_shape.0) {
        let t = [iw as f32 + 0.5, ih as f32 + 0.5];
        let mut alpha_sum = 0f32;
        let mut alpha_occu = 1f32;
        // draw front to back
        for idx in (0..idx2pnt.len()).rev() {
            let i_pnt = idx2pnt[idx];
            let splat2 = &pnt2splat2[i_pnt];
            if splat2.ndc_z() <= -1f32 || splat2.ndc_z() >= 1f32 { continue; }
            if !del_geo_core::aabb2::is_inlcude_point(splat2.aabb(), &[t[0], t[1]]) {
                continue;
            }
            let (pos_pix, sig_inv, rgb, alpha) = splat2.property();
            let t0 = [t[0] - pos_pix[0], t[1] - pos_pix[1]];
            let e = del_geo_core::mat2_sym::mult_vec_from_both_sides(sig_inv, &t0, &t0);
            let e = (-0.5 * e).exp() * alpha;
            let e_out = alpha_occu * e;
            img_data[(ih * img_shape.0 + iw) * 3 + 0] += rgb[0] * e_out;
            img_data[(ih * img_shape.0 + iw) * 3 + 1] += rgb[1] * e_out;
            img_data[(ih * img_shape.0 + iw) * 3 + 2] += rgb[2] * e_out;
            alpha_occu *= 1f32 - e;
            alpha_sum += e_out;
            if alpha_sum > 0.999 {
                break;
            }
        }
    }
    crate::write_png_from_float_image_rgb(path, &img_shape, &img_data)?;
    Ok(())
}
