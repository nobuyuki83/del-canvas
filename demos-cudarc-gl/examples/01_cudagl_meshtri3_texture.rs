use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoop;
use winit::window::Window;

#[cfg(feature = "cuda")]
use cudarc::driver::CudaSlice;

#[cfg(feature = "cuda")]
struct Content {
    pub tri2vtx: Vec<usize>,
    pub vtx2xyz: Vec<f32>,
    pub vtx2uv: Vec<f32>,
    pub dev: std::sync::Arc<cudarc::driver::CudaDevice>,
    // pub pix_to_tri: CudaFunction,
    pub tri2vtx_dev: CudaSlice<u32>,
    pub vtx2xyz_dev: CudaSlice<f32>,
    pub bvhnodes_dev: CudaSlice<u32>,
    pub aabbs_dev: CudaSlice<f32>,
    pub tex_shape: (usize, usize),
    pub tex_data: Vec<f32>,
}

#[cfg(feature = "cuda")]
impl del_gl_winit_glutin::app3::Content for Content {
    fn new() -> Self {
        let (tri2vtx, vtx2xyz, vtx2uv) = {
            let mut obj = del_msh_core::io_obj::WavefrontObj::<usize, f32>::new();
            obj.load("asset/spot/spot_triangulated.obj").unwrap();
            obj.unified_xyz_uv_as_trimesh()
        };
        let bvhnodes = del_msh_core::bvhnodes_morton::from_triangle_mesh(&tri2vtx, &vtx2xyz, 3);
        let aabbs = del_msh_core::bvhnode2aabb3::from_uniform_mesh_with_bvh(
            0,
            &bvhnodes,
            Some((&tri2vtx, 3)),
            &vtx2xyz,
            None,
        );
        //println!("{:?}",img.color());
        let (tex_data, tex_shape, bitdepth) =
            del_canvas_image::load_image_as_float_array("asset/spot/spot_texture.png").unwrap();
        assert_eq!(bitdepth, 3);
        let dev = cudarc::driver::CudaDevice::new(0).unwrap();
        // let pix_to_tri = dev.get_func("my_module", "pix_to_tri").unwrap();
        let tri2vtx_dev = dev
            .htod_copy(tri2vtx.iter().map(|&v| v as u32).collect())
            .unwrap();
        let vtx2xyz_dev = dev.htod_copy(vtx2xyz.clone()).unwrap();
        let bvhnodes_dev = dev
            .htod_copy(bvhnodes.iter().map(|&v| v as u32).collect())
            .unwrap();
        let aabbs_dev = dev.htod_copy(aabbs.clone()).unwrap();
        Self {
            tri2vtx,
            vtx2xyz,
            vtx2uv,
            dev,
            // pix_to_tri,
            tri2vtx_dev,
            vtx2xyz_dev,
            bvhnodes_dev,
            aabbs_dev,
            tex_data,
            tex_shape: tex_shape,
        }
    }

    fn compute_image(
        &mut self,
        img_shape: (usize, usize),
        cam_projection: &[f32; 16],
        cam_model: &[f32; 16],
    ) -> Vec<u8> {
        let now = std::time::Instant::now();
        let transform_world2ndc =
            del_geo_core::mat4_col_major::mult_mat(&cam_projection, &cam_model);
        let transform_ndc2world =
            del_geo_core::mat4_col_major::try_inverse(&transform_world2ndc).unwrap();
        let mut pix2tri_dev = self
            .dev
            .alloc_zeros::<u32>(img_shape.1 * img_shape.0)
            .unwrap();
        let transform_ndc2world_dev = self.dev.htod_copy(transform_ndc2world.to_vec()).unwrap();
        del_canvas_raycast_cudarc::pix2tri::pix2tri(
            &self.dev,
            img_shape,
            &mut pix2tri_dev,
            &self.tri2vtx_dev,
            &self.vtx2xyz_dev,
            &self.bvhnodes_dev,
            &self.aabbs_dev,
            &transform_ndc2world_dev,
        )
        .unwrap();
        let pix2tri = self.dev.dtoh_sync_copy(&pix2tri_dev).unwrap();
        println!("   Elapsed pix2tri: {:.2?}", now.elapsed());
        let now = std::time::Instant::now();
        /*
        for tri in pix2tri.iter() {
            if *tri != u32::MAX {
                dbg!(tri);
            }
        }
         */
        /*
        let pix2tri = del_canvas_core::raycast_trimesh3::pix2tri(
            &self.tri2vtx,
            &self.vtx2xyz,
            &self.bvhnodes,
            &self.aabbs,
            &img_size,
            &transform_ndc2world,
        );
         */
        let img_data = del_canvas_raycast::raycast_trimesh3::render_texture_from_pix2tri(
            img_shape,
            &transform_ndc2world,
            &self.tri2vtx,
            &self.vtx2xyz,
            &self.vtx2uv,
            &pix2tri,
            self.tex_shape,
            &self.tex_data,
            &del_canvas_image::texture::Interpolation::Bilinear,
        );
        let img_data: Vec<u8> = img_data
            .iter()
            .map(|v| (v * 255.0).clamp(0., 255.) as u8)
            .collect();
        println!("   Elapsed frag: {:.2?}", now.elapsed());
        img_data
    }
}

#[cfg(feature = "cuda")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let template = glutin::config::ConfigTemplateBuilder::new()
        .with_alpha_size(8)
        .with_transparency(cfg!(cgl_backend));
    let display_builder = {
        let window_attributes = Window::default_attributes()
            .with_transparent(false)
            .with_title("01_texture_fullscrn")
            .with_inner_size(PhysicalSize {
                width: 600,
                height: 600,
            });
        glutin_winit::DisplayBuilder::new().with_window_attributes(Some(window_attributes))
    };
    let mut app = del_gl_winit_glutin::app3::MyApp::<Content>::new(template, display_builder);
    let event_loop = EventLoop::new().unwrap();
    event_loop.run_app(&mut app)?;
    app.appi.exit_state
}

#[cfg(not(feature = "cuda"))]
fn main() {}
