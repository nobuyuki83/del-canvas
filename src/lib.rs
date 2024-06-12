use num_traits::AsPrimitive;

pub mod cam2;
pub mod cam3;
pub mod canvas_bitmap;
pub mod canvas_gif;
pub mod canvas_svg;
pub mod color;
pub mod colormap;
pub mod rasterize_circle;
pub mod rasterize_line;
pub mod rasterize_polygon;
pub mod raycast_trimesh2;
pub mod raycast_trimesh3;

fn hoge<Real>(p0: &[Real; 2], p1: &[Real; 2], p2: &[Real; 2], q: &[Real; 2]) -> Option<(Real, Real)>
where
    Real: num_traits::Float,
{
    let a0 = del_geo::tri2::area_(q, p1, p2);
    if a0 < Real::zero() {
        return None;
    }
    let a1 = del_geo::tri2::area_(q, p2, p0);
    if a1 < Real::zero() {
        return None;
    }
    let a2 = del_geo::tri2::area_(q, p0, p1);
    if a2 < Real::zero() {
        return None;
    }
    let sum_area_inv = Real::one() / (a0 + a1 + a2);
    Some((a0 * sum_area_inv, a1 * sum_area_inv))
}

#[allow(clippy::identity_op)]
pub fn triangle<Index, Real>(
    pix2color: &mut [u8],
    img_width: usize,
    p0: &[Real; 2],
    p1: &[Real; 2],
    p2: &[Real; 2],
    transform: &[Real; 9],
    i_color: u8,
) where
    Real: num_traits::Float + 'static + Copy + nalgebra::RealField,
    Index: AsPrimitive<usize>,
    usize: AsPrimitive<Real>,
{
    let half = Real::one() / (Real::one() + Real::one());
    let img_height = pix2color.len() / img_width;
    let q0: [Real; 2] = del_geo::mat3::transform_homogeneous(transform, p0).unwrap();
    let q1: [Real; 2] = del_geo::mat3::transform_homogeneous(transform, p1).unwrap();
    let q2: [Real; 2] = del_geo::mat3::transform_homogeneous(transform, p2).unwrap();
    for i_h in 0..img_height {
        for i_w in 0..img_width {
            let p_xy: [Real; 2] = [i_w.as_() + half, i_h.as_() + half];
            let Some((_r0, _r1)) = hoge(&q0, &q1, &q2, &p_xy) else {
                continue;
            };
            pix2color[i_h * img_width + i_w] = i_color;
        }
    }
}

pub fn write_png_from_float_image<Real, Path>(
    path: Path,
    img_width: usize,
    img_height: usize,
    img: &[Real],
) where
    Real: num_traits::Float + 'static + Copy + AsPrimitive<u8>,
    usize: AsPrimitive<Real>,
    Path: AsRef<std::path::Path>,
{
    let pix2color_u8: Vec<u8> = img.iter().map(|&v| (v * 255.as_()).as_()).collect();
    let file = std::fs::File::create(path).unwrap();
    let w = std::io::BufWriter::new(file);
    let mut encoder = png::Encoder::new(
        w,
        img_width.try_into().unwrap(),
        img_height.try_into().unwrap(),
    ); // Width is 2 pixels and height is 1.
    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(&pix2color_u8).unwrap(); // Save
}

#[test]
fn test_draw_mesh() {
    let (tri2vtx, vtx2xy) = del_msh::trimesh2_dynamic::meshing_from_polyloop2::<usize, f32>(
        &[0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0],
        0.11,
        0.11,
    );
    let vtx2color = {
        let mut vtx2color = vec![0.5_f32; vtx2xy.len() / 2];
        for i_vtx in 0..vtx2color.len() {
            vtx2color[i_vtx] = i_vtx as f32 / vtx2color.len() as f32;
        }
        vtx2color
    };
    dbg!(tri2vtx.len(), vtx2xy.len());
    let img_width = 400;
    let img_height = 300;
    let mut pix2color = vec![0_f32; img_width * img_height];
    let transform_xy2pix = nalgebra::Matrix3::<f32>::new(
        (img_width / 2) as f32,
        0.,
        0.,
        0.,
        -(img_height as f32),
        img_height as f32,
        0.,
        0.,
        1.,
    );
    crate::raycast_trimesh2::trimsh2_vtxcolor(
        img_width,
        img_height,
        &mut pix2color,
        &tri2vtx,
        &vtx2xy,
        &vtx2color,
        &transform_xy2pix,
    );
    write_png_from_float_image("target/hoge.png", img_width, img_height, &pix2color);
}
