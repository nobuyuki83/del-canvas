use ply_rs::ply::PropertyAccess;

pub fn hoge(scene: &pbrt4::Scene) -> (f32, [f32; 16], (usize, usize)) {
    let camera = scene.camera.as_ref().unwrap();
    let transform = camera.transform.to_cols_array();
    // dbg!(&camera.params);
    let fov = match camera.params {
        pbrt4::types::Camera::Perspective { fov, .. } => fov,
        _ => {
            todo!()
        }
    };
    let film = scene.film.as_ref().unwrap();
    let img_shape = (film.xresolution as usize, film.yresolution as usize);
    (fov, transform, img_shape)
}

pub fn trimesh3_from_shape_entity(
    shape_entity: &pbrt4::ShapeEntity,
    path_file: &str,
) -> Option<(usize, Option<usize>, Vec<usize>, Vec<f32>, Vec<f32>)> {
    match &shape_entity.params {
        pbrt4::types::Shape::TriangleMesh {
            positions,
            indices,
            normals,
            ..
        } => {
            let tri2vtx = indices.iter().map(|&v| v as usize).collect::<Vec<usize>>();
            return Some((
                shape_entity.material_index.unwrap(),
                shape_entity.area_light_index,
                tri2vtx,
                positions.to_vec(),
                normals.to_vec(),
            ));
        }
        pbrt4::types::Shape::PlyMesh { filename } => {
            let filename = filename.strip_suffix("\"").unwrap().to_string();
            let filename = filename.strip_prefix("\"").unwrap().to_string();
            let path = std::path::Path::new(path_file);
            let path = path.parent().unwrap();
            let path = path.to_str().unwrap().to_string() + "/" + &filename;
            let mut f = std::fs::File::open(path).unwrap();
            let p = ply_rs::parser::Parser::<ply_rs::ply::DefaultElement>::new();
            let ply = p.read_ply(&mut f).unwrap();
            let mut vtx2xyz: Vec<f32> = vec![];
            let mut tri2vtx: Vec<usize> = vec![];
            for (str, vals) in ply.payload {
                if str == "vertex" {
                    vtx2xyz.resize(vals.len() * 3, 0f32);
                    for (i_vtx, val) in vals.iter().enumerate() {
                        vtx2xyz[i_vtx * 3] = val.get_float(&"x".to_string()).unwrap();
                        vtx2xyz[i_vtx * 3 + 1] = val.get_float(&"y".to_string()).unwrap();
                        vtx2xyz[i_vtx * 3 + 2] = val.get_float(&"z".to_string()).unwrap();
                        /*
                        let nx = val.get_float(&"nx".to_string()).unwrap();
                        let ny = val.get_float(&"ny".to_string()).unwrap();
                        let nz = val.get_float(&"nz".to_string()).unwrap();
                        let u = val.get_float(&"u".to_string()).unwrap();
                        let v = val.get_float(&"v".to_string()).unwrap();
                         */
                    }
                } else if str == "face" {
                    tri2vtx.resize(vals.len() * 3, 0usize);
                    for (i_vtx, val) in vals.iter().enumerate() {
                        let a = val.get_list_int(&"vertex_indices".to_string()).unwrap();
                        tri2vtx[i_vtx * 3] = a[0] as usize;
                        tri2vtx[i_vtx * 3 + 1] = a[1] as usize;
                        tri2vtx[i_vtx * 3 + 2] = a[2] as usize;
                    }
                }
            }
            // TODO: parse normals for .ply
            return Some((
                shape_entity.material_index.unwrap(),
                shape_entity.area_light_index,
                tri2vtx,
                vtx2xyz,
                vec![],
            ));
        }
        _ => {
            panic!();
        }
    }
    None
}

pub fn spectrum_from_light_entity(area_light_entity: &pbrt4::types::AreaLight) -> Option<[f32; 3]> {
    match area_light_entity {
        pbrt4::types::AreaLight::Diffuse {
            filename: _filename,
            two_sided: _two_sided,
            spectrum,
            scale: _scale,
        } => match spectrum.unwrap().to_owned() {
            pbrt4::param::Spectrum::Rgb(rgb) => Some(rgb),
            _ => {
                panic!();
            }
        },
    }
}
