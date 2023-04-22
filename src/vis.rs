extern crate gl;
extern crate glutin;
use crate::gl_wrap::{set_attrib, Bind, Drop, Program, UniformMatrix};
use crate::globe::Globe;
use crate::mouse::{rotate_from_mouse, zoom_from_scroll, MouseState};
use glam::{Mat4, Vec3};
use glutin::dpi::{LogicalSize, PhysicalPosition};
use glutin::event::{ElementState, Event, MouseButton, MouseScrollDelta, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::{Window, WindowBuilder};
use glutin::{ContextBuilder, ContextWrapper, GlRequest, PossiblyCurrent};

// wrapper for initialization and running vis
pub struct Vis {
    gl: VisGl,
    window: VisWindow,
}

impl Vis {
    pub fn new(width: f64, height: f64) -> Result<Self, VisError> {
        // initialize gl ctx and window
        let window = VisWindow::new(width, height)?;
        // setup vis gl resources
        let gl = VisGl::new(width, height)?;
        Ok(Self { gl, window })
    }

    // vis as argument since run requires move
    pub fn start(vis: Vis) -> Result<(), VisError> {
        VisWindow::run(vis.window, vis.gl)?;
        Ok(())
    }
}

// contains all vis logic and gl resources
struct VisGl {
    pub globe: Globe,
    pub mvp: MvpMatrices,
    pub mouse: MouseState,
}

impl VisGl {
    pub fn new(width: f64, height: f64) -> Result<Self, VisError> {
        let mouse = MouseState::new();
        let globe = Globe::new()?;
        let mvp = MvpMatrices::new_default(&globe.program, (width / height) as f32)?;
        Ok(Self { globe, mvp, mouse })
    }

    fn mouse_move(&mut self, position: PhysicalPosition<f64>) {
        if self.mouse.dragging {
            let dx = position.x - self.mouse.x;
            let dy = position.y - self.mouse.y;
            // rotate model matrix from mouse move deltas
            self.mvp.model.data = rotate_from_mouse(self.mvp.model.data, dx, dy);
            self.mvp.model.apply();
        }
        // save last mouse position
        self.mouse.x = position.x;
        self.mouse.y = position.y;
    }

    fn mouse_input(&mut self, button: MouseButton, state: ElementState) {
        // save mouse drag state on left mouse input
        if let MouseButton::Left = button {
            self.mouse.dragging = match state {
                ElementState::Pressed => true,
                ElementState::Released => false,
            }
        }
    }

    fn mouse_wheel(&mut self, delta: MouseScrollDelta) {
        let ds = match delta {
            MouseScrollDelta::PixelDelta(position) => position.y,
            MouseScrollDelta::LineDelta(_, y) => y as f64,
        };
        // zoom view matrix from mouse scroll delta
        self.mvp.view.data = zoom_from_scroll(self.mvp.view.data, ds);
        self.mvp.view.apply();
    }

    // get main draw loop as closure
    pub fn get_draw() -> impl FnMut(&mut VisGl) {
        let mut globe_draw = Globe::get_draw();
        move |vis: &mut VisGl| {
            unsafe {
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            }
            globe_draw(&mut vis.globe)
        }
    }

    // bind required resources for start of draw loop
    pub fn setup_gl_resources(&self) -> Result<(), VisError> {
        self.globe.program.bind();
        self.globe.buffer.bind();
        set_attrib(&self.globe.program, "position", 3, 3, 0)?;

        self.mvp.proj.apply();
        self.mvp.view.apply();
        self.mvp.model.apply();
        Ok(())
    }
}

impl Drop for VisGl {
    fn drop(&self) {
        self.globe.drop();
    }
}

// contains gl context, window, event loop
struct VisWindow {
    pub ctx: ContextWrapper<PossiblyCurrent, Window>,
    pub event_loop: EventLoop<()>,
}

impl VisWindow {
    pub fn new(width: f64, height: f64) -> Result<Self, VisError> {
        let window = WindowBuilder::new()
            .with_inner_size(LogicalSize::new(width, height))
            .with_title("window");
        let event_loop = EventLoop::new();
        let ctx_builder = ContextBuilder::new()
            .with_gl(GlRequest::GlThenGles {
                // version 2.0 for WebGL compatibility
                opengl_version: (2, 0),
                opengles_version: (2, 0),
            })
            .with_multisampling(4)
            .build_windowed(window, &event_loop)?;
        let ctx;
        unsafe {
            ctx = ctx_builder.make_current().unwrap();
            gl::load_with(|ptr| ctx.get_proc_address(ptr) as *const _);
            gl::Enable(gl::DEPTH_TEST);
        }
        Ok(Self { ctx, event_loop })
    }

    // window passed as argument since running event loop causes move
    // calls vis event handlers on event
    pub fn run(window: VisWindow, mut vis: VisGl) -> Result<(), VisError> {
        vis.setup_gl_resources()?;
        let mut draw = VisGl::get_draw();
        window.event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CursorMoved { position, .. } => {
                        vis.mouse_move(position);
                    }
                    WindowEvent::MouseWheel { delta, .. } => {
                        vis.mouse_wheel(delta);
                    }
                    WindowEvent::MouseInput { button, state, .. } => {
                        vis.mouse_input(button, state);
                    }
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    _ => (),
                },
                Event::LoopDestroyed => {
                    vis.drop();
                }
                Event::RedrawRequested(_) => {
                    draw(&mut vis);
                    window.ctx.swap_buffers().unwrap();
                    window.ctx.window().request_redraw();
                }
                _ => (),
            }
        });
    }
}

// matrices for 3D scene
// one instance for all programs, same matrices used everywhere
struct MvpMatrices {
    pub proj: UniformMatrix,
    pub view: UniformMatrix,
    pub model: UniformMatrix,
}

impl MvpMatrices {
    // initialize matrices with default values
    pub fn new_default(program: &Program, aspect: f32) -> Result<Self, MvpError> {
        let proj = UniformMatrix::new(
            program,
            "projMatrix",
            Mat4::perspective_rh_gl(1.25, aspect, 0.1, 10.0),
        )?;
        let view = UniformMatrix::new(
            program,
            "viewMatrix",
            Mat4::look_at_rh(Vec3::new(0.0, 0.0, 2.0), Vec3::ZERO, Vec3::Y),
        )?;
        let model = UniformMatrix::new(program, "modelMatrix", Mat4::IDENTITY)?;
        Ok(Self { proj, view, model })
    }
}

use thiserror::Error;

#[derive(Error, Debug)]
pub enum VisError {
    #[error("{0}")]
    Mvp(#[from] MvpError),
    #[error("{0}")]
    Globe(#[from] crate::globe::GlobeError),
    #[error("{0}")]
    Nul(#[from] std::ffi::NulError),
    #[error("{0}")]
    CtxCreation(#[from] glutin::CreationError),
}

#[derive(Error, Debug)]
pub enum MvpError {
    #[error("{0}")]
    UniformMatrix(#[from] crate::gl_wrap::UniformMatrixError),
}
