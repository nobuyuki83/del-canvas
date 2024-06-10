fn main() {
    let mut canvas = del_canvas::canvas_gif::Canvas::new(
        "target/lloyd.gif",
        (300, 300),
        &vec![0xffffff, 0x000000, 0xff0000],
    );
    let transform_to_scr = nalgebra::Matrix3::<f32>::new(
        canvas.width as f32 * 0.8,
        0.,
        canvas.width as f32 * 0.1,
        0.,
        -(canvas.height as f32) * 0.8,
        canvas.height as f32 * 0.9,
        0.,
        0.,
        1.,
    );
    // loop vtxs
    let vtxl2xy = vec![0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0];
    // site vtxs
    let mut site2xy = del_msh::sampling::poisson_disk_sampling_from_polyloop2(&vtxl2xy, 0.08, 100);
    for _iter in 0..50 {
        canvas.clear(0);
        let (site2vtxc2xy, _site2vtxc2info) =
            { del_msh::voronoi2::voronoi_cells(&vtxl2xy, &site2xy) };
        let mut site2xy_new = del_msh::vtx2xyz::to_array_of_nalgebra_vector(&site2xy);
        for i_site in 0..site2xy.len() / 2 {
            let vtxc2xy = &site2vtxc2xy[i_site];
            let vtxc2xy = del_msh::vtx2xyz::from_array_of_nalgebra(&vtxc2xy);
            let p = del_msh::polyloop2::cog_as_face(&vtxc2xy);
            site2xy_new[i_site] = p.into();
            del_canvas::paint_pixcenter::polyloop(
                &mut canvas.data,
                canvas.width,
                &vtxc2xy,
                &transform_to_scr,
                1.,
                1,
            );
            del_canvas::paint_pixcenter::point(
                &mut canvas.data,
                canvas.width,
                &[site2xy[i_site * 2 + 0], site2xy[i_site * 2 + 1]],
                &transform_to_scr,
                3.0,
                1,
            );
            del_canvas::paint_pixcenter::point(
                &mut canvas.data,
                canvas.width,
                &[p[0], p[1]],
                &transform_to_scr,
                3.0,
                2,
            );
        }
        canvas.write();
        site2xy = del_msh::vtx2xyz::from_array_of_nalgebra(&site2xy_new);
    }
}
