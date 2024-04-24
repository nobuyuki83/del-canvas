use num_traits::AsPrimitive;

pub mod canvas;
pub mod canvas_gif;
pub mod colormap;


///
/// * `transform` - from `xy` to `pixel coordinate`
pub fn trimsh2_vtxcolor<Index, Real>(
    img_width: usize,
    img_height: usize,
    pix2color: &mut [Real],
    tri2vtx: &[Index],
    vtx2xy: &[Real],
    vtx2color: &[Real],
    transform: &nalgebra::Matrix3<Real>)
    where Real: num_traits::Float + 'static + Copy + nalgebra::RealField,
          Index: AsPrimitive<usize>,
          usize: AsPrimitive<Real>
{
    let num_dim = pix2color.len() / (img_width * img_height);
    let num_vtx = vtx2xy.len() / 2;
    let transform_inv = transform.clone().try_inverse().unwrap();
    assert_eq!(vtx2color.len(), num_vtx * num_dim);
    for i_h in 0..img_height {
        for i_w in 0..img_width {
            let p_xy = transform_inv * nalgebra::Vector3::<Real>::new(
                i_w.as_(), i_h.as_(), Real::one());
            let p_xy = [p_xy[0] / p_xy[2], p_xy[1] / p_xy[2]];
            let Some((i_tri, r0, r1))
                = del_msh::trimesh2::search_bruteforce_one_triangle_include_input_point(
                &p_xy, tri2vtx, vtx2xy) else { continue; };
            let r2 = Real::one() - r0 - r1;
            let iv0: usize = tri2vtx[i_tri*3+0].as_();
            let iv1: usize = tri2vtx[i_tri*3+1].as_();
            let iv2: usize = tri2vtx[i_tri*3+2].as_();
            for i_dim in 0..num_dim {
                pix2color[(i_h * img_width + i_w) * num_dim + i_dim]
                    = r0 * vtx2color[iv0 * num_dim + i_dim]
                    + r1 * vtx2color[iv1 * num_dim + i_dim]
                    + r2 * vtx2color[iv2 * num_dim + i_dim];
            }
        }
    }
}

pub fn write_png_from_float_image<Real, Path>(
    path: Path,
    img_width: usize,
    img_height: usize,
    img: &[Real])
where Real: num_traits::Float + 'static + Copy + AsPrimitive<u8>,
    usize: AsPrimitive<Real>,
    Path: AsRef<std::path::Path>
{
    let pix2color_u8: Vec<u8> = img.iter().map(|&v| (v * 255.as_()).as_() ).collect();
    let file = std::fs::File::create(path).unwrap();
    let w = std::io::BufWriter::new(file);
    let mut encoder = png::Encoder::new(
        w,
        img_width.try_into().unwrap(),
        img_height.try_into().unwrap()); // Width is 2 pixels and height is 1.
    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(&pix2color_u8).unwrap(); // Save
}

#[test]
fn test_draw_mesh() {
    let (tri2vtx, vtx2xy)
        = del_msh::trimesh2_dynamic::meshing_from_polyloop2::<usize, f32>(
        &[0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0], 0.11);
    let vtx2color = {
        let mut vtx2color = vec!(0.5_f32; vtx2xy.len() / 2);
        for i_vtx in 0..vtx2color.len() {
            vtx2color[i_vtx] = i_vtx as f32 / vtx2color.len() as f32;
        }
        vtx2color
    };
    dbg!(tri2vtx.len(), vtx2xy.len());
    let img_width = 400;
    let img_height = 300;
    let mut pix2color = vec!(0_f32; img_width * img_height);
    let transform_xy2pix = nalgebra::Matrix3::<f32>::new(
        (img_width/2) as f32, 0., 0.,
        0., -(img_height as f32), img_height as f32,
        0., 0., 1.);
    trimsh2_vtxcolor(
        img_width, img_height, &mut pix2color,
        &tri2vtx, &vtx2xy, &vtx2color, &transform_xy2pix);
    write_png_from_float_image("target/hoge.png", img_width, img_height, &pix2color);
}