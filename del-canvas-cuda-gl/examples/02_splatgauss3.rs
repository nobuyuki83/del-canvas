use cudarc::driver::{CudaFunction, CudaSlice, DeviceSlice};
use del_gl_core::gl;
use del_gl_core::gl::types::GLfloat;
use del_winit_glutin::app_internal;
use glutin::display::GlDisplay;
//
use cudarc::driver::LaunchAsync;
use image::EncodableLayout;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{KeyEvent, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{Key, NamedKey};
use winit::window::Window;
use del_canvas_cuda::splat_gauss::Splat2;

pub struct MyApp {
    pub appi: crate::app_internal::AppInternal,
    pub renderer: Option<del_gl_core::drawer_array_xyzuv::Drawer>,
    //
    pub dev: std::sync::Arc<cudarc::driver::CudaDevice>,
    pub pnt2splat3_dev: CudaSlice<del_canvas_cuda::splat_gauss::Splat3>,
    pub pnt2splat2_dev: CudaSlice<del_canvas_cuda::splat_gauss::Splat2>,
    pub pix2rgb_dev: CudaSlice<f32>,
    //
    pub view_rot: del_geo_core::view_rotation::Trackball,
    pub view_prj: del_geo_core::view_projection::Perspective,
    pub ui_state: del_gl_core::view_ui_state::UiState,
}

impl MyApp {
    pub fn new(
        template: glutin::config::ConfigTemplateBuilder,
        display_builder: glutin_winit::DisplayBuilder,
    ) -> Self {
        let file_path = "../asset/dog.ply";
        let pnt2splat3 = del_msh_core::io_ply::read_3d_gauss_splat::<_, del_canvas_cuda::splat_gauss::Splat3>(file_path).unwrap();
        //println!("{:?}",img.color());

        let dev = cudarc::driver::CudaDevice::new(0).unwrap();
        let pnt2splat2_dev = {
            let pnt2splat2 = vec![Splat2::default(); pnt2splat3.len()];
            dev.htod_copy(pnt2splat2.clone()).unwrap()
        };
        let pnt2splat3_dev = dev.htod_copy(pnt2splat3).unwrap();
        let pix2rgb_dev = dev.alloc_zeros::<f32>(1).unwrap();
        //
        Self {
            appi: app_internal::AppInternal::new(template, display_builder),
            renderer: None,
            dev,
            // pix_to_tri,
            pnt2splat3_dev,
            pnt2splat2_dev,
            pix2rgb_dev,
            ui_state: del_gl_core::view_ui_state::UiState::new(),
            view_rot: del_geo_core::view_rotation::Trackball::new(),
            view_prj: del_geo_core::view_projection::Perspective {
                lens: 24.,
                near: 0.5,
                far: 3.0,
                cam_pos: [0., 0., 2.],
                proj_direction: true,
                scale: 1.,
            },
        }
    }
}

impl ApplicationHandler for MyApp {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        use glutin::display::GetGlDisplay;
        let Some(app_state) = self.appi.resumed(event_loop) else {
            return;
        };
        // The context needs to be current for the Renderer to set up shaders and
        // buffers. It also performs function loading, which needs a current context on
        // WGL.
        self.renderer.get_or_insert_with(|| {
            let gl_display = &app_state.gl_context.display();
            let gl = gl::Gl::load_with(|symbol| {
                let symbol = std::ffi::CString::new(symbol).unwrap();
                gl_display.get_proc_address(symbol.as_c_str()).cast()
            });
            let mut render = del_gl_core::drawer_array_xyzuv::Drawer::new(gl);
            render.init_gl();
            render
        });
        unsafe {
            //
            let Some(rndr) = &self.renderer else {
                panic!();
            };
            let gl = &rndr.gl;
            {
                #[rustfmt::skip]
                static VERTEX_DATA: [f32; 24] = [
                    -1.0, -1.0, 0., 1.,
                    1.0, -1.0, 1., 1.,
                    1.0, 1.0, 1., 0.,
                    //
                    -1.0, -1.0, 0., 1.,
                    1.0, 1.0, 1., 0.,
                    -1.0, 1.0, 0., 0.
                ];
                gl.BindBuffer(gl::ARRAY_BUFFER, rndr.vbo);
                gl.BufferData(
                    gl::ARRAY_BUFFER,
                    (VERTEX_DATA.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                    VERTEX_DATA.as_ptr() as *const _,
                    gl::STATIC_DRAW,
                );
            }
        }
        assert!(self.appi.state.replace(app_state).is_none());
    }

    fn suspended(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        // This event is only raised on Android, where the backing NativeWindow for a GL
        // Surface can appear and disappear at any moment.
        println!("Android window removed");
        self.appi.suspended();
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        use glutin::prelude::GlSurface;
        match event {
            WindowEvent::Resized(size) if size.width != 0 && size.height != 0 => {
                // Some platforms like EGL require resizing GL surface to update the size
                // Notable platforms here are Wayland and macOS, other don't require it
                // and the function is no-op, but it's wise to resize it for portability
                // reasons.
                if let Some(app_internal::AppState {
                                gl_context,
                                gl_surface,
                                window: _,
                            }) = self.appi.state.as_ref()
                {
                    gl_surface.resize(
                        gl_context,
                        std::num::NonZeroU32::new(size.width).unwrap(),
                        std::num::NonZeroU32::new(size.height).unwrap(),
                    );
                    let renderer = self.renderer.as_ref().unwrap();
                    renderer.resize(size.width as i32, size.height as i32);
                    self.ui_state.win_width = size.width;
                    self.ui_state.win_height = size.height;
                }
            }
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event:
                KeyEvent {
                    logical_key: Key::Named(NamedKey::Escape),
                    ..
                },
                ..
            } => event_loop.exit(),
            _ => (),
        }
        let redraw = del_winit_glutin::view_navigation(
            event,
            &mut self.ui_state,
            &mut self.view_prj,
            &mut self.view_rot,
        );
        if redraw {
            if let Some(state) = &self.appi.state {
                state.window.request_redraw();
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        use glutin::prelude::GlSurface;
        if let Some(app_internal::AppState {
                        gl_context,
                        gl_surface,
                        window,
                    }) = self.appi.state.as_ref()
        {
            let now = std::time::Instant::now();
            let img_shape = {
                (
                    window.inner_size().width as usize,
                    window.inner_size().height as usize,
                )
            };
            let cam_model = self.view_rot.mat4_col_major();
            let cam_projection = self
                .view_prj
                .mat4_col_major(img_shape.0 as f32 / img_shape.1 as f32);
            let transform_world2ndc =
                del_geo_core::mat4_col_major::mult_mat(&cam_projection, &cam_model);
            let transform_world2ndc_dev = self.dev.htod_copy(transform_world2ndc.to_vec()).unwrap();
            del_canvas_cuda::splat_gauss::pnt2splat3_to_pnt2splat2(
                &self.dev,
                &self.pnt2splat3_dev,
                &mut self.pnt2splat2_dev,
                &transform_world2ndc_dev,
                (img_shape.0 as u32, img_shape.1 as u32),
            ).unwrap();
            let tile_size = 16usize;
            let (tile2idx_dev, idx2pnt_dev) = del_canvas_cuda::splat_gauss::tile2idx_idx2pnt(
                &self.dev,
                (img_shape.0 as u32, img_shape.1 as u32), tile_size as u32,
                &self.pnt2splat2_dev).unwrap();
            if self.pix2rgb_dev.len() != img_shape.0 * img_shape.1 * 3 {
                self.pix2rgb_dev = self.dev.alloc_zeros::<f32>(img_shape.0 * img_shape.1 * 3).unwrap();
            }
            self.dev.memset_zeros(&mut self.pix2rgb_dev);
            del_canvas_cuda::splat_gauss::rasterize_pnt2splat2(
                &self.dev, (img_shape.0 as u32, img_shape.1 as u32),
                &mut self.pix2rgb_dev,
                &self.pnt2splat2_dev,
                tile_size as u32,
                &tile2idx_dev,
                &idx2pnt_dev).unwrap();
            let img_data = self.dev.dtoh_sync_copy(&self.pix2rgb_dev).unwrap();
            assert_eq!(img_data.len(), img_shape.0 * img_shape.1 * 3);
            let img_data: Vec<u8> = img_data
                .iter()
                .map(|v| (v * 255.0).clamp(0., 255.) as u8)
                .collect();
            println!("   Elapsed frag: {:.2?}", now.elapsed());
            //println!("{:?}",img.color());
            let Some(ref rndr) = self.renderer else {
                panic!();
            };
            let gl = &rndr.gl;
            unsafe {
                gl.BindTexture(gl::TEXTURE_2D, rndr.id_tex);
                gl.PixelStorei(gl::UNPACK_ALIGNMENT, 1);
                gl.TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RGB.try_into().unwrap(),
                    img_shape.0.try_into().unwrap(),
                    img_shape.1.try_into().unwrap(),
                    0,
                    gl::RGB,
                    gl::UNSIGNED_BYTE,
                    img_data.as_ptr() as *const _,
                );
                gl.GenerateMipmap(gl::TEXTURE_2D);
            }
            //
            let renderer = self.renderer.as_ref().unwrap();
            renderer.draw();
            window.request_redraw();
            gl_surface.swap_buffers(gl_context).unwrap();
        }
    }
}

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
    let mut app = MyApp::new(template, display_builder);
    let event_loop = EventLoop::new().unwrap();
    event_loop.run_app(&mut app)?;
    app.appi.exit_state
}
