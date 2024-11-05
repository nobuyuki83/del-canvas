struct Splat2 {
    ndc_z: f32,
    rad_pix: f32,
    pos_pix: [f32; 2],
    rgb: [f32; 3],
}

impl del_canvas_cpu::splat_circle::Splat2 for Splat2 {
    fn ndc_z(&self) -> f32 {
        self.ndc_z
    }
    fn property(&self) -> (&[f32; 2], f32, &[f32; 3]) {
        (&self.pos_pix, self.rad_pix, &self.rgb)
    }
}

fn main() -> anyhow::Result<()> {
    let path_dir: String =
        "C:/Users/nobuy/Downloads/small_data_colmap/colmap_data/sparse/0/".to_string();
    let cameras = del_canvas_cpu::colmap::read_cameras(path_dir.clone() + "cameras.bin")?;
    let images = del_canvas_cpu::colmap::read_images(path_dir.clone() + "images.bin")?;
    let points3d = del_canvas_cpu::colmap::read_points3d(path_dir.clone() + "points3D.bin")?;
    let pntid2pntidx = {
        let maxid = points3d.iter().map(|v| v.id).max().unwrap();
        let mut pntid2pntidx = vec![-1i32; maxid as usize + 1];
        for (ip, p) in points3d.iter().enumerate() {
            let id = p.id;
            pntid2pntidx[id as usize] = ip as i32;
        }
        pntid2pntidx
    };
    for img0 in images.iter() {
        // let img0 = &images[0];
        let camera_id = img0.camera_id;
        let camera = cameras.iter().find(|&v| v.camera_id == camera_id).unwrap();
        assert_eq!(camera.camera_id, camera_id);
        let transform_camin = match camera.model {
            del_canvas_cpu::colmap::CameraModel::PINHOLE(param) => [
                param[0], 0f64, 0f64, 0f64, param[1], 0f64, param[2], param[3], 1f64,
            ],
            _ => {
                panic!()
            }
        };
        use del_geo_core::mat4_col_major;
        let quat = [img0.qvec[1], img0.qvec[2], img0.qvec[3], img0.qvec[0]];
        let mat_rot = del_geo_core::quaternion::to_mat4_col_major(&quat);
        let mat_transl = mat4_col_major::from_translate(&img0.tvec);
        let mat0 = mat4_col_major::mult_mat(&mat_transl, &mat_rot);
        let img_shape = (camera.width as usize, camera.height as usize);
        let mut pnt2splat2: Vec<Splat2> = vec![];
        for (ivtx, &pntid) in img0.vtx2id.iter().enumerate() {
            let xy0 = &img0.vtx2xy[ivtx * 2..ivtx * 2 + 2];
            if pntid == -1 {
                continue;
            }
            assert!((pntid as usize) < pntid2pntidx.len());
            let pntidx = pntid2pntidx[pntid as usize];
            assert_ne!(pntidx, -1);
            assert!((pntidx as usize) < points3d.len());
            let point = &points3d[pntidx as usize];
            let xyz_world = &point.xyz;
            let xyz_cam = mat4_col_major::transform_homogeneous(&mat0, xyz_world).unwrap();
            let a = del_geo_core::mat3_col_major::mult_vec(&transform_camin, &xyz_cam);
            let xy1 = [a[0] / a[2], a[1] / a[2]];
            let dist = del_geo_core::edge2::length(&[xy0[0], xy0[1]], &xy1);
            // dbg!(dist);
            assert!(dist < 15.0);
        }
        for point in points3d.iter() {
            let xyz_world = &point.xyz;
            let xyz_cam = mat4_col_major::transform_homogeneous(&mat0, xyz_world).unwrap();
            let a = del_geo_core::mat3_col_major::mult_vec(&transform_camin, &xyz_cam);
            let xy1 = [a[0] / a[2], a[1] / a[2]];
            pnt2splat2.push(Splat2 {
                pos_pix: [xy1[0] as f32, xy1[1] as f32],
                ndc_z: -(a[2] / 10.0) as f32,
                rad_pix: 5.,
                rgb: [
                    (point.rgb[0] as f32) / 255.0,
                    (point.rgb[1] as f32) / 255.0,
                    (point.rgb[2] as f32) / 255.0,
                ],
            });
        }
        let name = img0.name.trim_end();
        let name = name.split('.').collect::<Vec<&str>>()[0];
        let fname = format!("target/del_canvas_cpu__colmap__{}.png", name);
        dbg!(&fname);
        del_canvas_cpu::splat_circle::draw_sort_z(&pnt2splat2, img_shape, &fname)?;
    }
    Ok(())
}
