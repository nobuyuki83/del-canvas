pub enum Interpolation {
    Nearest,
    Bilinear,
}

/// coordinate (0., 0.) is the center ot the texel
pub fn bilinear_integer_center<const NDIM: usize>(
    pix: &[f32; 2],
    tex_shape: &(usize, usize),
    tex_data: &[f32],
) -> [f32; NDIM] {
    let rx = pix[0] - pix[0].floor();
    let ix0 = pix[0].floor() as i64;
    if ix0 < 0 || ix0 >= tex_shape.0 as i64 {
        return [0f32; NDIM];
    }
    let ry = pix[1] - pix[1].floor();
    let iy0 = pix[1].floor() as i64;
    if iy0 < 0 || iy0 >= tex_shape.1 as i64 {
        return [0f32; NDIM];
    }
    //
    let ix0 = ix0 as usize;
    let iy0 = iy0 as usize;
    let ix1 = ix0 + 1;
    let iy1 = iy0 + 1;
    let i00_tex = iy0 * tex_shape.0 + ix0;
    let i10_tex = iy0 * tex_shape.0 + ix1;
    let i01_tex = iy1 * tex_shape.0 + ix0;
    let i11_tex = iy1 * tex_shape.0 + ix1;
    std::array::from_fn(|i_dim| {
        let v00 = tex_data[i00_tex * NDIM + i_dim];
        let v01 = tex_data[i01_tex * NDIM + i_dim];
        let v10 = tex_data[i10_tex * NDIM + i_dim];
        let v11 = tex_data[i11_tex * NDIM + i_dim];
        (1. - rx) * (1. - ry) * v00 + rx * (1. - ry) * v10 + (1. - rx) * ry * v01 + rx * ry * v11
    })
}

/// coordinate (0., 0.) is the center ot the texel
pub fn nearest<const NDIM: usize>(
    pix: &[f32; 2],
    tex_shape: &(usize, usize),
    tex_data: &[f32],
    is_integer_center: bool,
) -> [f32; NDIM] {
    let x0 = if is_integer_center {
        pix[0] + 0.5
    } else {
        pix[0]
    };
    let y0 = if is_integer_center {
        pix[1] + 0.5
    } else {
        pix[1]
    };
    let ix0 = x0 as i64;
    let iy0 = y0 as i64;
    if ix0 < 0 || ix0 >= tex_shape.0 as i64 {
        return [0f32; NDIM];
    }
    if iy0 < 0 || iy0 >= tex_shape.1 as i64 {
        return [0f32; NDIM];
    }
    let i_tex = (iy0 as usize) * tex_shape.0 + (ix0 as usize);
    std::array::from_fn(|i_dim| tex_data[i_tex * NDIM + i_dim])
}
