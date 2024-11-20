fn main() {
    let mut reng = rand::thread_rng();
    let mut canvas = del_canvas_image::canvas_gif::Canvas::new(
        "target/lloyd.gif",
        (300, 300),
        &vec![0xffffff, 0x000000, 0xff0000],
    );
    let transform_world2pix = del_canvas_cpu::cam2::transform_world2pix_ortho_preserve_asp(
        &(canvas.width, canvas.height),
        &[-0.3, -0.1, 1.3, 1.1],
    );
    // loop vtxs
    let vtxl2xy = vec![0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0];
    // site vtxs
    let mut site2xy = del_msh_core::sampling::poisson_disk_sampling_from_polyloop2(
        &vtxl2xy, 0.08, 100, &mut reng,
    );
    for _iter in 0..50 {
        canvas.clear(0);
        let site2cell = { del_msh_nalgebra::voronoi2::voronoi_cells(&vtxl2xy, &site2xy, |_v| true) };
        let mut site2xy_new = del_msh_core::vtx2xdim::to_array_of_nalgebra_vector(&site2xy);
        for i_site in 0..site2xy.len() / 2 {
            let vtxc2xy = &site2cell[i_site].vtx2xy;
            let vtxc2xy = del_msh_core::vtx2xdim::from_array_of_nalgebra(&vtxc2xy);
            let p = del_msh_core::polyloop2::cog_as_face(&vtxc2xy);
            site2xy_new[i_site] = p.into();
            del_canvas_cpu::rasterize_polygon::stroke(
                &mut canvas.data,
                canvas.width,
                &vtxc2xy,
                &transform_world2pix,
                1.,
                1,
            );
            del_canvas_cpu::rasterize_circle::fill(
                &mut canvas.data,
                canvas.width,
                &[site2xy[i_site * 2 + 0], site2xy[i_site * 2 + 1]],
                &transform_world2pix,
                3.0,
                1,
            );
            del_canvas_cpu::rasterize_circle::fill(
                &mut canvas.data,
                canvas.width,
                &[p[0], p[1]],
                &transform_world2pix,
                3.0,
                2,
            );
        }
        canvas.write();
        site2xy = del_msh_core::vtx2xdim::from_array_of_nalgebra(&site2xy_new);
    }
}
