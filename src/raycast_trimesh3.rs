use num_traits::AsPrimitive;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;

pub fn pix2tri<Index>(
    tri2vtx: &[Index],
    vtx2xyz: &[f32],
    bvhnodes: &[Index],
    aabbs: &[f32],
    img_shape: &(usize, usize), // (width, height)
    transform_ndc2world: &[f32; 16],
) -> Vec<Index>
where
    Index: num_traits::PrimInt + AsPrimitive<usize> + Sync + Send,
    usize: AsPrimitive<Index>,
{
    let tri_for_pix = |i_pix: usize| {
        let i_h = i_pix / img_shape.0;
        let i_w = i_pix - i_h * img_shape.0;
        //
        let (ray_org, ray_dir) =
            crate::cam3::ray3_homogeneous((i_w, i_h), img_shape, transform_ndc2world);
        let mut hits: Vec<(f32, usize)> = vec![];
        del_msh_core::bvh3::search_intersection_ray::<Index>(
            &mut hits,
            &ray_org,
            &ray_dir,
            &del_msh_core::bvh3::TriMeshWithBvh {
                tri2vtx,
                vtx2xyz,
                bvhnodes,
                aabbs,
            },
            0,
        );
        hits.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        let Some(&(_depth, i_tri)) = hits.first() else {
            return Index::max_value();
        };
        i_tri.as_()
    };
    let img: Vec<Index> = (0..img_shape.0 * img_shape.1)
        .into_par_iter()
        .map(tri_for_pix)
        .collect();
    img
}

pub fn render_depth_bvh(
    image_size: (usize, usize),
    img_data: &mut [f32],
    transform_ndc2world: &[f32; 16],
    tri2vtx: &[usize],
    vtx2xyz: &[f32],
    bvhnodes: &[usize],
    aabbs: &[f32],
) {
    let transform_world2ndc: [f32; 16] =
        nalgebra::Matrix4::<f32>::from_column_slice(transform_ndc2world)
            .try_inverse()
            .unwrap()
            .as_slice()
            .try_into()
            .unwrap();
    let (width, height) = image_size;
    for ih in 0..height {
        for iw in 0..width {
            let (ray_org, ray_dir) =
                crate::cam3::ray3_homogeneous((iw, ih), &image_size, transform_ndc2world);
            let mut hits: Vec<(f32, usize)> = vec![];
            del_msh_core::bvh3::search_intersection_ray(
                &mut hits,
                &ray_org,
                &ray_dir,
                &del_msh_core::bvh3::TriMeshWithBvh {
                    tri2vtx,
                    vtx2xyz,
                    bvhnodes,
                    aabbs,
                },
                0,
            );
            hits.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            let Some(&(depth, _i_tri)) = hits.first() else {
                continue;
            };
            let pos = del_geo_core::vec3::axpy(depth, &ray_dir, &ray_org);
            let ndc =
                del_geo_core::mat4_col_major::transform_homogeneous(&transform_world2ndc, &pos)
                    .unwrap();
            let depth_ndc = (ndc[2] + 1f32) * 0.5f32;
            img_data[ih * width + iw] = depth_ndc;
        }
    }
}

pub fn render_normalmap_pix2tri(
    image_size: (usize, usize),
    cam_modelviewd: &[f32; 16],
    tri2vtx: &[usize],
    vtx2xyz: &[f32],
    pix2tri: &[usize],
) -> Vec<f32> {
    let (width, height) = image_size;
    let mut img = vec![0f32; height * width * 3];
    for ih in 0..height {
        for iw in 0..width {
            let i_tri = pix2tri[ih * width + iw];
            if i_tri == usize::MAX {
                continue;
            }
            let tri = del_msh_core::trimesh3::to_tri3(i_tri, tri2vtx, vtx2xyz);
            let nrm = tri.normal();
            let nrm = del_geo_core::mat4_col_major::transform_vector(cam_modelviewd, &nrm);
            let unrm = del_geo_core::vec3::normalized(&nrm);
            img[(ih * width + iw) * 3 + 0] = unrm[0] * 0.5 + 0.5;
            img[(ih * width + iw) * 3 + 1] = unrm[1] * 0.5 + 0.5;
            img[(ih * width + iw) * 3 + 2] = unrm[2] * 0.5 + 0.5;
        }
    }
    img
}
