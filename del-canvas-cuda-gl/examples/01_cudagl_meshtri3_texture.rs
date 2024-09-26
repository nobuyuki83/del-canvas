use cudarc::driver::{CudaFunction, CudaSlice};
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

pub struct MyApp {
    pub appi: crate::app_internal::AppInternal,
    pub renderer: Option<del_gl_core::drawer_array_xyzuv::Drawer>,
    pub tri2vtx: Vec<usize>,
    pub vtx2xyz: Vec<f32>,
    pub vtx2uv: Vec<f32>,
    pub dev: std::sync::Arc<cudarc::driver::CudaDevice>,
    // pub pix_to_tri: CudaFunction,
    pub tri2vtx_dev: CudaSlice<u32>,
    pub vtx2xyz_dev: CudaSlice<f32>,
    pub bvhnodes_dev: CudaSlice<u32>,
    pub aabbs_dev: CudaSlice<f32>,
    //
    pub tex_shape: (usize, usize),
    pub tex_data: Vec<f32>,
    pub view_rot: del_geo_core::view_rotation::Trackball,
    pub view_prj: del_geo_core::view_projection::Perspective,
    pub ui_state: del_gl_core::view_ui_state::UiState,
}

impl MyApp {
    pub fn new(
        template: glutin::config::ConfigTemplateBuilder,
        display_builder: glutin_winit::DisplayBuilder,
    ) -> Self {
        let (tri2vtx, vtx2xyz, vtx2uv) = {
            let mut obj = del_msh_core::io_obj::WavefrontObj::<usize, f32>::new();
            obj.load("../asset/spot_triangulated.obj").unwrap();
            obj.unified_xyz_uv_as_trimesh()
        };
        let bvhnodes = del_msh_core::bvhnodes_morton::from_triangle_mesh(&tri2vtx, &vtx2xyz, 3);
        let aabbs = del_msh_core::aabbs3::from_uniform_mesh_with_bvh(
            0,
            &bvhnodes,
            Some((&tri2vtx, 3)),
            &vtx2xyz,
            None,
        );
        //println!("{:?}",img.color());
        let (tex_data, tex_shape, bitdepth) =
            del_canvas_cpu::load_image_as_float_array("../asset/spot_texture.png").unwrap();
        assert_eq!(bitdepth, 3);
        let dev = cudarc::driver::CudaDevice::new(0).unwrap();
        dev.load_ptx(
            del_canvas_cuda_kernel::PIX2TRI.into(),
            "my_module",
            &["pix_to_tri"],
        )
        .unwrap();
        // let pix_to_tri = dev.get_func("my_module", "pix_to_tri").unwrap();
        let tri2vtx_dev = dev
            .htod_copy(tri2vtx.iter().map(|&v| v as u32).collect())
            .unwrap();
        let vtx2xyz_dev = dev.htod_copy(vtx2xyz.clone()).unwrap();
        let bvhnodes_dev = dev
            .htod_copy(bvhnodes.iter().map(|&v| v as u32).collect())
            .unwrap();
        let aabbs_dev = dev.htod_copy(aabbs.clone()).unwrap();
        //
        Self {
            appi: app_internal::AppInternal::new(template, display_builder),
            renderer: None,
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
            let transform_ndc2world =
                del_geo_core::mat4_col_major::try_inverse(&transform_world2ndc).unwrap();
            let mut pix2tri_dev = self
                .dev
                .alloc_zeros::<u32>(img_shape.1 * img_shape.0)
                .unwrap();
            let transform_ndc2world_dev = self.dev.htod_copy(transform_ndc2world.to_vec()).unwrap();
            del_canvas_cuda::pix2tri::pix2tri(
                &self.dev,
                img_shape,
                &mut pix2tri_dev,
                &self.tri2vtx_dev,
                &self.vtx2xyz_dev,
                &self.bvhnodes_dev,
                &self.aabbs_dev,
                &transform_ndc2world_dev).unwrap();
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
            let img_data = del_canvas_cpu::raycast_trimesh3::render_texture_from_pix2tri(
                img_shape,
                &transform_ndc2world,
                &self.tri2vtx,
                &self.vtx2xyz,
                &self.vtx2uv,
                &pix2tri,
                self.tex_shape,
                &self.tex_data,
                &del_canvas_cpu::texture::Interpolation::Bilinear,
            );
            let img_data: Vec<u8> = img_data
                .iter()
                .map(|v| (v * 255.0).clamp(0., 255.) as u8)
                .collect();
            println!("   Elapsed frag: {:.2?}", now.elapsed());
            assert_eq!(img_data.len(), img_shape.0 * img_shape.1 * 3);
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
                width: 600,
                height: 600,
            });
        glutin_winit::DisplayBuilder::new().with_window_attributes(Some(window_attributes))
    };
    let mut app = MyApp::new(template, display_builder);
    let event_loop = EventLoop::new().unwrap();
    event_loop.run_app(&mut app)?;
    app.appi.exit_state
}
