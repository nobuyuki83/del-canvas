pub mod cam2;
pub mod cam3;
pub mod canvas_bitmap;
pub mod canvas_gif;
pub mod canvas_svg;
pub mod color;
pub mod colormap;
pub mod morphology;
pub mod rasterize;
pub mod texture;

use num_traits::AsPrimitive;

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

pub fn write_hdr_file<P>(
    path_output: P,
    img_shape: (usize, usize),
    img: &[f32],
) -> anyhow::Result<()>
where P: AsRef<std::path::Path>
{
    // write output
    let file1 = std::fs::File::create(path_output)?;
    use image::codecs::hdr::HdrEncoder;
    let enc = HdrEncoder::new(file1);
    let img: &[image::Rgb<f32>] =
        unsafe { std::slice::from_raw_parts(img.as_ptr() as _, img.len() / 3) };
    let _ = enc.encode(&img, img_shape.0, img_shape.1);
    Ok(())
}

pub fn write_hdr_file_mse_rgb_error_map(
    target_file: String,
    img_shape: (usize, usize),
    ground_truth: &[f32],
    img: &[f32],
) {
    assert_eq!(img.len(), img_shape.0 * img_shape.1 * 3);
    assert_eq!(ground_truth.len(), img_shape.0 * img_shape.1 * 3);
    let err = |a: &[f32], b: &[f32]| -> image::Rgb<f32> {
        let sq = (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2);
        image::Rgb([sq; 3])
    };
    let img_error: Vec<image::Rgb<f32>> = img
        .chunks(3)
        .zip(ground_truth.chunks(3))
        .map(|(a, b)| err(a, b))
        .collect();
    use image::codecs::hdr::HdrEncoder;
    let file = std::fs::File::create(target_file).unwrap();
    let enc = HdrEncoder::new(file);
    let _ = enc.encode(&img_error, img_shape.0, img_shape.1);
}

pub fn rmse_error(gt: &[f32], rhs: &[f32]) -> f32 {
    let up: f32 = gt
        .iter()
        .zip(rhs.iter())
        .map(|(&l, &r)| (l - r) * (l - r))
        .sum();
    let down: f32 = gt.iter().map(|&v| v * v).sum();
    up / down
}

pub fn expand_image(
    (img_width_in, img_height_in): (usize, usize),
    img_data_in: &[f32],
    ratio: usize,
) -> ((usize, usize), Vec<f32>) {
    let img_width_out = img_width_in * ratio;
    let img_height_out = img_height_in * ratio;
    let mut img_data_out = vec![0f32; img_height_out * img_width_out * 3];
    for iwi in 0..img_width_in {
        for ihi in 0..img_height_in {
            let pix_in =
                &img_data_in[(ihi * img_width_in + iwi) * 3..(ihi * img_width_in + iwi) * 3 + 3];
            for k in 0..ratio {
                for l in 0..ratio {
                    let iwo = iwi * ratio + k;
                    let iho = ihi * ratio + l;
                    let pix_out = &mut img_data_out
                        [(iho * img_width_out + iwo) * 3..(iho * img_width_out + iwo) * 3 + 3];
                    pix_out.copy_from_slice(pix_in);
                }
            }
        }
    }
    ((img_width_out, img_height_out), img_data_out)
}