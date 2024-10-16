use std::io::BufReader;
use byteorder::{LittleEndian, ReadBytesExt};
use num_traits::AsPrimitive;

pub enum CameraModel {
    PINHOLE([f64;4]),
}

pub struct Camera {
    pub camera_id: i32,
    pub width: u64,
    pub height: u64,
    pub model: CameraModel
}

pub fn read_cameras<PATH: AsRef<std::path::Path>>(path: PATH) -> anyhow::Result<Vec<Camera>>
{
    // read intrinsic
    let file = std::fs::File::open(path).unwrap();
    let mut reader = std::io::BufReader::new(file);
    let num_cam = reader.read_u64::<LittleEndian>()?;
    dbg!(num_cam);
    let mut cameras: Vec<Camera> = vec!();
    for _ in 0..num_cam {
        let camera_id = reader.read_i32::<LittleEndian>()?;
        let model_id = reader.read_u32::<LittleEndian>()?;
        let width = reader.read_u64::<LittleEndian>()?;
        let height = reader.read_u64::<LittleEndian>()?;
        dbg!(camera_id, model_id, width, height);
        let model = match model_id {
            1 => {
                let mut v = [0f64;4];
                for i_param in 0..4 {
                    v[i_param] = reader.read_f64::<LittleEndian>()?;
                }
                CameraModel::PINHOLE(v)
            },
            _ => { panic!() }
        };
        cameras.push(
            Camera {
                camera_id,
                height,
                width,
                model
            }
        );
    }
    Ok(cameras)
}

pub fn parse_string(reader: &mut BufReader<std::fs::File>) -> anyhow::Result<String>{
    let mut aa: Vec<u8> = vec!();
    loop {
        let a0 = reader.read_u8()?;
        aa.push(a0);
        if a0 == 0x00 {
            break;
        }
    }
    Ok( String::from_utf8(aa).unwrap() )
}

pub struct Image {
    pub image_id: i32,
    pub qvec: [f64;4],
    pub tvec: [f64;3],
    pub camera_id: i32,
    pub name: String,
    pub vtx2xy: Vec<f64>,
    pub vtx2id: Vec<i64>
}

pub fn read_images<PATH: AsRef<std::path::Path>>(path: PATH) -> anyhow::Result<Vec<Image>>
{
    // read intrinsic
    let file = std::fs::File::open(path).unwrap();
    let mut reader = std::io::BufReader::new(file);
    let num_images = reader.read_u64::<LittleEndian>()?;
    let mut images =  Vec::<Image>::with_capacity(num_images.as_());
    for _ in 0..num_images {
        let image_id = reader.read_i32::<LittleEndian>()?;
        let qvec = [
            reader.read_f64::<LittleEndian>()?,
            reader.read_f64::<LittleEndian>()?,
            reader.read_f64::<LittleEndian>()?,
            reader.read_f64::<LittleEndian>()?];
        let tvec = [
            reader.read_f64::<LittleEndian>()?,
            reader.read_f64::<LittleEndian>()?,
            reader.read_f64::<LittleEndian>()?];
        let camera_id = reader.read_i32::<LittleEndian>()?;
        let name = parse_string(&mut reader)?;
        let num_points = reader.read_u64::<LittleEndian>()?;
        let mut vtx2xy: Vec<f64> = vec!();
        let mut vtx2id: Vec<i64> = vec!();
        for _i_points in 0..num_points {
            let x = reader.read_f64::<LittleEndian>()?;
            let y = reader.read_f64::<LittleEndian>()?;
            let id_s = reader.read_i64::<LittleEndian>()?;
            vtx2xy.push(x);
            vtx2xy.push(y);
            vtx2id.push(id_s);
        }
        images.push( Image {
            image_id,
            qvec,
            tvec,
            camera_id,
            name,
            vtx2xy,
            vtx2id
        } );
    }
    Ok(images)
}

pub struct Point3D {
    pub id: u64,
    pub xyz: [f64;3],
    pub rgb: [u8;3],
    pub err: f64,
    pub img: Vec<(i32,i32)>
}

pub fn read_points3d<PATH: AsRef<std::path::Path>>(path: PATH) -> anyhow::Result<Vec<Point3D>>
{
    // read intrinsic
    let file = std::fs::File::open(path).unwrap();
    let mut reader = std::io::BufReader::new(file);
    let num_points = reader.read_u64::<LittleEndian>()?;
    let mut points = Vec::<Point3D>::with_capacity(num_points.as_());
    // dbg!(num_points);
    for _i_point in 0..num_points {
        let point3d_id =  reader.read_u64::<LittleEndian>()?;
        let x =  reader.read_f64::<LittleEndian>()?;
        let y =  reader.read_f64::<LittleEndian>()?;
        let z =  reader.read_f64::<LittleEndian>()?;
        let r =  reader.read_u8()?;
        let g =  reader.read_u8()?;
        let b =  reader.read_u8()?;
        let error = reader.read_f64::<LittleEndian>()?;
        let track_length = reader.read_u64::<LittleEndian>()?;
        // dbg!(point3d_id, x, y, z, error);
        // dbg!(track_length);
        let mut img = vec!((0i32, 0i32);0);
        for _i_track in 0..track_length {
            let image_id = reader.read_i32::<LittleEndian>()?;
            let point2d_idx = reader.read_i32::<LittleEndian>()?;
            // dbg!(image_id, point2d_idx);
            img.push((image_id, point2d_idx));
        }
        points.push(
            Point3D {
                id: point3d_id,
                xyz: [x,y,z],
                rgb: [r,g,b],
                err: error,
                img
            }
        )
    }
    Ok(points)
}