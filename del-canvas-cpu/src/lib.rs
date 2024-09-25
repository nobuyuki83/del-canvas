use num_traits::AsPrimitive;

pub mod cam2;
pub mod cam3;
pub mod canvas_bitmap;
pub mod canvas_gif;
pub mod canvas_svg;
pub mod color;
pub mod colormap;
pub mod morphology;
pub mod rasterize_aabb3;
pub mod rasterize_circle;
pub mod rasterize_line;
pub mod rasterize_polygon;
pub mod rasterize_triangle;
pub mod raycast_trimesh2;
pub mod raycast_trimesh3;
pub mod splat_circlez;
pub mod splat_gaussian2z;
pub mod splat_point2;
pub mod splat_point2z;
pub mod splat_point3;
pub mod texture;
pub mod tile_acceleration;

pub fn write_png_from_float_image_grayscale<Real, Path>(
    path: Path,
    img_shape: &(usize, usize),
    img: &[Real],
) -> anyhow::Result<()>
where
    Real: num_traits::Float + 'static + Copy + AsPrimitive<u8>,
    usize: AsPrimitive<Real>,
    Path: AsRef<std::path::Path>,
{
    let pix2color_u8: Vec<u8> = img
        .iter()
        .map(|&v| {
            let a: Real = v.clamp(Real::zero(), Real::one());
            (a * 255.as_()).as_()
        })
        .collect();
    let file = std::fs::File::create(path)?;
    let w = std::io::BufWriter::new(file);
    let mut encoder = png::Encoder::new(
        w,
        img_shape.0.try_into().unwrap(),
        img_shape.1.try_into().unwrap(),
    ); // Width is 2 pixels and height is 1.
    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(&pix2color_u8)?; // Save
    Ok(())
}

pub fn write_png_from_float_image_rgb<Real, Path>(
    path: Path,
    img_shape: &(usize, usize),
    img: &[Real],
) -> anyhow::Result<()>
where
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
    let file = std::fs::File::create(path)?;
    let w = std::io::BufWriter::new(file);
    let mut encoder = png::Encoder::new(w, img_shape.0.try_into()?, img_shape.1.try_into()?); // Width is 2 pixels and height is 1.
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(&pix2color_u8)?; // Save
    Ok(())
}

pub fn load_image_as_float_array<P>(path: P) -> anyhow::Result<(Vec<f32>, (usize, usize), usize)>
where
    P: AsRef<std::path::Path>,
{
    use image::GenericImageView;
    let img_trg = image::open(path)?;
    let (width, height) = img_trg.dimensions();
    let (width, height) = (width as usize, height as usize);
    let depth: usize = img_trg.color().bytes_per_pixel().into();
    let img_trg = img_trg.into_bytes();
    let img_trg: Vec<f32> = img_trg.iter().map(|&v| (v as f32) / 255.0f32).collect();
    assert_eq!(img_trg.len(), width * height * depth);
    Ok((img_trg, (width, height), depth))
}
