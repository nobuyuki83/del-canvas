pub fn rasterize(
    point2gauss: &[f32],
    point2splat: &[f32],
    tile2jdx: &[usize],
    jdx2idx: &[usize],
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
    for ih in 0..img_shape.1 {
        for iw in 0..img_shape.0 {
            let tile_shape: (usize, usize) = (img_shape.0 / tile_size, img_shape.1 / tile_size);
            let i_tile = (ih / tile_size) * tile_shape.0 + (iw / tile_size);
            let t = nalgebra::Vector2::<f32>::new(iw as f32 + 0.5, ih as f32 + 0.5);
            let mut alpha_sum = 0f32;
            let mut alpha_occu = 1f32;
            for &idx in &jdx2idx[tile2jdx[i_tile]..tile2jdx[i_tile + 1]] {
                let i_point = idx2point[idx];
                let pos_pix = arrayref::array_ref![point2splat, i_point * NDOF_SPLAT, 2];
                let abc = arrayref::array_ref![point2splat, i_point * NDOF_SPLAT + 2, 3];
                let aabb = arrayref::array_ref![point2splat, i_point * NDOF_SPLAT + 5, 4];
                let w = nalgebra::Matrix2::<f32>::new(abc[0], abc[1], abc[1], abc[2]);
                let pos_pix = nalgebra::Vector2::<f32>::from_column_slice(pos_pix);
                let color = arrayref::array_ref![point2gauss, i_point * NDOF_GAUSS + 3, 3];
                if !del_geo_core::aabb2::is_inlcude_point(&aabb, &[t[0], t[1]]) {
                    continue;
                }
                let t0 = t - pos_pix;
                let e = (t0.transpose() * w * t0).x;
                let e = (-0.5 * e).exp();
                let e_out = alpha_occu * e;
                img_data[(ih * img_shape.0 + iw) * 3 + 0] += color[0] * e_out;
                img_data[(ih * img_shape.0 + iw) * 3 + 1] += color[1] * e_out;
                img_data[(ih * img_shape.0 + iw) * 3 + 2] += color[2] * e_out;
                alpha_occu *= 1f32 - e;
                alpha_sum += e_out;
                if alpha_sum > 0.999 {
                    break;
                }
            }
        }
    }
    img_data
}

pub fn diff_point2gauss(
    point2gauss: &[f32],
    point2splat: &[f32],
    tile2jdx: &[usize],
    jdx2idx: &[usize],
    idx2point: &[usize],
    img_shape: (usize, usize),
    tile_size: usize,
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
    for ih in 0..img_shape.1 {
        for iw in 0..img_shape.0 {
            let dw_rgb = arrayref::array_ref!(dw_pix2rgb, (ih * img_shape.0 + iw) * 3, 3);
            let tile_shape: (usize, usize) = (img_shape.0 / tile_size, img_shape.1 / tile_size);
            let i_tile = (ih / tile_size) * tile_shape.0 + (iw / tile_size);
            let pos_center = nalgebra::Vector2::<f32>::new(iw as f32 + 0.5, ih as f32 + 0.5);
            let mut alpha_sum = 0f32;
            let mut alpha_occu = 1f32;
            for &idx in &jdx2idx[tile2jdx[i_tile]..tile2jdx[i_tile + 1]] {
                let i_point = idx2point[idx];
                let pos_splat = arrayref::array_ref![point2splat, i_point * NDOF_SPLAT, 2];
                let abc = arrayref::array_ref![point2splat, i_point * NDOF_SPLAT + 2, 3];
                let aabb = arrayref::array_ref![point2splat, i_point * NDOF_SPLAT + 5, 4];
                let w = nalgebra::Matrix2::<f32>::new(abc[0], abc[1], abc[1], abc[2]);
                let pos_splat = nalgebra::Vector2::<f32>::from_column_slice(pos_splat);
                let color = arrayref::array_ref![point2gauss, i_point * NDOF_GAUSS + 3, 3];
                if !del_geo_core::aabb2::is_inlcude_point(&aabb, &[pos_center[0], pos_center[1]]) {
                    continue;
                }
                let t0 = pos_center - pos_splat;
                let e = (t0.transpose() * w * t0).x;
                let e = (-0.5 * e).exp();
                let e_out = alpha_occu * e;
                dw_point2gauss[i_point * NDOF_GAUSS + 3] += dw_rgb[0] * e_out;
                dw_point2gauss[i_point * NDOF_GAUSS + 4] += dw_rgb[0] * e_out;
                dw_point2gauss[i_point * NDOF_GAUSS + 5] += dw_rgb[0] * e_out;
                //dw_point2gauss[(ih * img_shape.0 + iw) * 3 + 0] += color[0] * e_out;
                //dw_point2gauss[(ih * img_shape.0 + iw) * 3 + 1] += color[1] * e_out;
                //dw_point2gauss[(ih * img_shape.0 + iw) * 3 + 2] += color[2] * e_out;
                alpha_occu *= 1f32 - e;
                alpha_sum += e_out;
                if alpha_sum > 0.999 {
                    break;
                }
            }
        }
    }
    dw_point2gauss
}
