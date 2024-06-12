use num_traits::AsPrimitive;

///
/// * `transform` - from `xy` to `pixel coordinate`
#[allow(clippy::identity_op)]
pub fn trimsh2_vtxcolor<Index, Real>(
    img_width: usize,
    img_height: usize,
    pix2color: &mut [Real],
    tri2vtx: &[Index],
    vtx2xy: &[Real],
    vtx2color: &[Real],
    transform: &nalgebra::Matrix3<Real>,
) where
    Real: num_traits::Float + 'static + Copy + nalgebra::RealField,
    Index: AsPrimitive<usize>,
    usize: AsPrimitive<Real>,
{
    let num_dim = pix2color.len() / (img_width * img_height);
    let num_vtx = vtx2xy.len() / 2;
    let transform_inv = transform.clone().try_inverse().unwrap();
    assert_eq!(vtx2color.len(), num_vtx * num_dim);
    for i_h in 0..img_height {
        for i_w in 0..img_width {
            let p_xy =
                transform_inv * nalgebra::Vector3::<Real>::new(i_w.as_(), i_h.as_(), Real::one());
            let p_xy = [p_xy[0] / p_xy[2], p_xy[1] / p_xy[2]];
            let Some((i_tri, r0, r1)) =
                del_msh::trimesh2::search_bruteforce_one_triangle_include_input_point(
                    &p_xy, tri2vtx, vtx2xy,
                )
            else {
                continue;
            };
            let r2 = Real::one() - r0 - r1;
            let iv0: usize = tri2vtx[i_tri * 3 + 0].as_();
            let iv1: usize = tri2vtx[i_tri * 3 + 1].as_();
            let iv2: usize = tri2vtx[i_tri * 3 + 2].as_();
            for i_dim in 0..num_dim {
                pix2color[(i_h * img_width + i_w) * num_dim + i_dim] = r0
                    * vtx2color[iv0 * num_dim + i_dim]
                    + r1 * vtx2color[iv1 * num_dim + i_dim]
                    + r2 * vtx2color[iv2 * num_dim + i_dim];
            }
        }
    }
}
