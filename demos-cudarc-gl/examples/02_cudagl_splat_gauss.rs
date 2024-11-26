#[cfg(feature = "cuda")]
use cudarc::driver::{CudaSlice, DeviceSlice};
#[cfg(feature = "cuda")]
use del_canvas_splat_cudarc::splat_gauss::Splat2;

use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoop;
use winit::window::Window;
//

#[cfg(feature = "cuda")]
pub struct Content {
    pub dev: std::sync::Arc<cudarc::driver::CudaDevice>,
    pub pnt2splat3_dev: CudaSlice<del_canvas_splat_cudarc::splat_gauss::Splat3>,
    pub pnt2splat2_dev: CudaSlice<del_canvas_splat_cudarc::splat_gauss::Splat2>,
    pub pix2rgb_dev: CudaSlice<f32>,
}

#[cfg(feature = "cuda")]
impl del_gl_winit_glutin::app3::Content for Content {
    fn new() -> Self {
        let file_path = "asset/dog.ply";
        //let file_path = "C:/Users/nobuy/Downloads/ChilliPepperPlant.ply"; //"asset/dog.ply";
        //let file_path = "C:/Users/nobuy/Downloads/bread.ply"; //"asset/dog.ply";
        //let file_path = "C:/Users/nobuy/Downloads/PumpkinTree.ply"; //"asset/dog.ply";
        //let file_path = "C:/Users/nobuy/Downloads/plant.ply"; //"asset/dog.ply";
        // let pnt2splat3 = del_msh_core::io_ply::read_3d_gauss_splat::<_, del_canvas_cuda::splat_gauss::Splat3>(file_path).unwrap();
        //println!("{:?}",img.color());
        let pnt2splat3 = {
            let mut pnt2splat3 = del_msh_core::io_ply::read_3d_gauss_splat::<
                _,
                del_canvas_splat_cudarc::splat_gauss::Splat3,
            >(file_path)
            .unwrap();
            let aabb3 = del_msh_core::vtx2point::aabb3_from_points(&pnt2splat3);
            let longest_edge = del_geo_core::aabb3::max_edge_size(&aabb3);
            let scale = 1.5 / longest_edge;
            let center = del_geo_core::aabb3::center(&aabb3);
            pnt2splat3.iter_mut().for_each(|s| {
                s.xyz[0] -= center[0];
                s.xyz[1] -= center[1];
                s.xyz[2] -= center[2];
                s.xyz[0] *= scale;
                s.xyz[1] *= scale;
                s.xyz[2] *= scale;
                s.scale[0] *= scale;
                s.scale[1] *= scale;
                s.scale[2] *= scale;
            });
            pnt2splat3
        };

        let dev = cudarc::driver::CudaDevice::new(0).unwrap();
        let pnt2splat2_dev = {
            let pnt2splat2 = vec![Splat2::default(); pnt2splat3.len()];
            dev.htod_copy(pnt2splat2.clone()).unwrap()
        };
        let pnt2splat3_dev = dev.htod_copy(pnt2splat3).unwrap();
        let pix2rgb_dev = dev.alloc_zeros::<f32>(1).unwrap();
        //
        Self {
            dev,
            // pix_to_tri,
            pnt2splat3_dev,
            pnt2splat2_dev,
            pix2rgb_dev,
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
        let transform_world2ndc_dev = self.dev.htod_copy(transform_world2ndc.to_vec()).unwrap();
        del_canvas_splat_cudarc::splat_gauss::pnt2splat3_to_pnt2splat2(
            &self.dev,
            &self.pnt2splat3_dev,
            &mut self.pnt2splat2_dev,
            &transform_world2ndc_dev,
            (img_shape.0 as u32, img_shape.1 as u32),
        )
        .unwrap();
        let tile_size = 16usize;
        let (tile2idx_dev, idx2pnt_dev) = del_canvas_splat_cudarc::splat_gauss::tile2idx_idx2pnt(
            &self.dev,
            (img_shape.0 as u32, img_shape.1 as u32),
            tile_size as u32,
            &self.pnt2splat2_dev,
        )
        .unwrap();
        if self.pix2rgb_dev.len() != img_shape.0 * img_shape.1 * 3 {
            self.pix2rgb_dev = self
                .dev
                .alloc_zeros::<f32>(img_shape.0 * img_shape.1 * 3)
                .unwrap();
        }
        self.dev.memset_zeros(&mut self.pix2rgb_dev).unwrap();
        del_canvas_splat_cudarc::splat_gauss::rasterize_pnt2splat2(
            &self.dev,
            (img_shape.0 as u32, img_shape.1 as u32),
            &mut self.pix2rgb_dev,
            &self.pnt2splat2_dev,
            tile_size as u32,
            &tile2idx_dev,
            &idx2pnt_dev,
        )
        .unwrap();
        let img_data = self.dev.dtoh_sync_copy(&self.pix2rgb_dev).unwrap();
        assert_eq!(img_data.len(), img_shape.0 * img_shape.1 * 3);
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
                width: 16 * 30,
                height: 16 * 30,
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
