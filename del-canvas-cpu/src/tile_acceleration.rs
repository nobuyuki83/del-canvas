use num_traits::AsPrimitive;

pub trait Splat2 {
    fn aabb(&self) -> [f32; 4];
    fn ndc_z(&self) -> f32;
}

pub fn tile2pnt<S: Splat2, INDEX>(
    pnt2circle: &[S],
    img_shape: (usize, usize),
    tile_size: usize,
) -> (Vec<INDEX>, Vec<INDEX>)
where
    INDEX: num_traits::PrimInt + std::ops::AddAssign<INDEX> + AsPrimitive<usize>,
    usize: AsPrimitive<INDEX>,
{
    assert_eq!(img_shape.0 % tile_size, 0);
    assert_eq!(img_shape.0 % tile_size, 0);
    let tile_shape = (img_shape.0 / tile_size, img_shape.1 / tile_size);
    let num_tile = tile_shape.0 * tile_shape.1;
    let mut tile2ind = vec![INDEX::zero(); num_tile + 1];
    for i_vtx in 0..pnt2circle.len() {
        let aabb2 = pnt2circle[i_vtx].aabb(); //del_geo_core::aabb2::from_point(&p0, rad);
        let tiles = del_geo_core::aabb2::overlapping_tiles(&aabb2, tile_size, tile_shape);
        for &i_tile in tiles.iter() {
            tile2ind[i_tile + 1] += INDEX::one();
        }
    }
    for i_tile in 0..num_tile {
        let ind0 = tile2ind[i_tile + 1];
        tile2ind[i_tile + 1] = tile2ind[i_tile] + ind0;
    }
    let num_ind: usize = tile2ind[num_tile].as_();
    let mut ind2vtx = vec![INDEX::zero(); num_ind];
    let mut ind2tiledepth = Vec::<(usize, usize, f32)>::with_capacity(num_ind);
    for i_vtx in 0..pnt2circle.len() {
        let aabb2 = pnt2circle[i_vtx].aabb();
        let depth = pnt2circle[i_vtx].ndc_z();
        let tiles = del_geo_core::aabb2::overlapping_tiles(&aabb2, tile_size, tile_shape);
        for &i_tile in tiles.iter() {
            ind2vtx[ind2tiledepth.len()] = i_vtx.as_();
            ind2tiledepth.push((i_vtx, i_tile, -depth));
        }
    }
    assert_eq!(ind2tiledepth.len(), num_ind);
    ind2tiledepth.sort_by(|&a, &b| {
        if a.1 == b.1 {
            a.2.partial_cmp(&b.2).unwrap()
        } else {
            a.1.cmp(&b.1)
        }
    });
    for iind in 0..ind2tiledepth.len() {
        ind2vtx[iind] = ind2tiledepth[iind].0.as_();
    }
    (tile2ind, ind2vtx)
}
