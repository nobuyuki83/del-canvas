use num_traits::AsPrimitive;

pub mod cam2;
pub mod cam3;
pub mod canvas_bitmap;
pub mod canvas_gif;
pub mod canvas_svg;
pub mod color;
pub mod colormap;
pub mod rasterize_circle;
pub mod rasterize_gaussian_splatting;
pub mod rasterize_line;
pub mod rasterize_points3;
pub mod rasterize_polygon;
pub mod rasterize_triangle;
pub mod raycast_trimesh2;
pub mod raycast_trimesh3;

pub fn write_png_from_float_image_grayscale<Real, Path>(
    path: Path,
    img_shape: &(usize, usize),
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
        img_shape.0.try_into().unwrap(),
        img_shape.1.try_into().unwrap(),
    ); // Width is 2 pixels and height is 1.
    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(&pix2color_u8).unwrap(); // Save
}

pub fn write_png_from_float_image_rgb<Real, Path>(
    path: Path,
    img_shape: &(usize, usize),
    img: &[Real],
) where
    Real: num_traits::Float + 'static + Copy + AsPrimitive<u8>,
    usize: AsPrimitive<Real>,
    Path: AsRef<std::path::Path>,
{
    let zero = Real::zero();
    let one = Real::one();
    let v255: Real = 255usize.as_();
    let pix2color_u8: Vec<u8> = img
        .iter()
        .map(|&v| {
            let a: Real = v.clamp(zero, one);
            (a * v255).as_()
        })
        .collect();
    let file = std::fs::File::create(path).unwrap();
    let w = std::io::BufWriter::new(file);
    let mut encoder = png::Encoder::new(
        w,
        img_shape.0.try_into().unwrap(),
        img_shape.1.try_into().unwrap(),
    ); // Width is 2 pixels and height is 1.
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(&pix2color_u8).unwrap(); // Save
}
