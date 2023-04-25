use crate::gl_wrap::{set_attrib, Bind, Drop, Program, UniformMatrix};
use crate::globe::Globe;
use crate::mouse::{rotate_from_mouse, zoom_from_scroll, MouseButton, MouseState};
use glam::{Mat4, Vec3};
use glow::HasContext;

#[cfg(not(target_arch = "wasm32"))]
mod native {
    pub use glutin::dpi::LogicalSize;
    pub use glutin::event::MouseButton as MouseButtonGlutin;
    pub use glutin::event::{ElementState, Event, MouseScrollDelta, WindowEvent};
    pub use glutin::event_loop::ControlFlow;
    pub use glutin::event_loop::EventLoop;
    pub use glutin::window::WindowBuilder;
    pub use glutin::ContextBuilder;
}

#[cfg(target_arch = "wasm32")]
mod web {
    pub use wasm_bindgen::JsCast;
    pub use web_sys::{window, HtmlCanvasElement, WebGl2RenderingContext};
}

// wrapper for initialization and running vis
pub struct Vis {
    gl: VisGl,
    window: VisContext,
}

impl Vis {
    pub fn new(width: f64, height: f64) -> Result<Self, VisError> {
        // initialize gl ctx and window
        let window = VisContext::new(width, height)?;
        // setup vis gl resources
        let gl = VisGl::new(&window, width, height)?;
        Ok(Self { gl, window })
    }

    // vis as argument since run requires move
    pub fn start(vis: Vis) -> Result<(), VisError> {
        VisContext::run(vis.window, vis.gl)?;
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
    pub fn new(context: &VisContext, width: f64, height: f64) -> Result<Self, VisError> {
        let mouse = MouseState::new();
        let globe = Globe::new(&context.gl, &context.shader_version)?;
        let mvp = MvpMatrices::new_default(&context.gl, &globe.program, (width / height) as f32)?;
        Ok(Self { globe, mvp, mouse })
    }

    fn mouse_move(&mut self, gl: &glow::Context, x: f64, y: f64) {
        if self.mouse.dragging {
            let dx = x - self.mouse.x;
            let dy = y - self.mouse.y;
            // rotate model matrix from mouse move deltas
            self.mvp.model.data = rotate_from_mouse(self.mvp.model.data, dx, dy);
            self.mvp.model.apply(gl);
        }
        // save last mouse position
        self.mouse.x = x;
        self.mouse.y = y;
    }

    fn mouse_input(&mut self, _: &glow::Context, button: MouseButton, pressed: bool) {
        // save mouse drag state on left mouse input
        if let MouseButton::Left = button {
            self.mouse.dragging = pressed;
        }
    }

    fn mouse_wheel(&mut self, gl: &glow::Context, delta: f64) {
        self.mvp.view.data = zoom_from_scroll(self.mvp.view.data, delta);
        self.mvp.view.apply(gl);
    }

    // get main draw loop as closure
    pub fn get_draw() -> impl FnMut(&glow::Context, &mut VisGl) {
        let mut globe_draw = Globe::get_draw();
        move |gl: &glow::Context, vis: &mut VisGl| {
            unsafe {
                gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
            }
            globe_draw(gl, &mut vis.globe)
        }
    }

    // bind required resources for start of draw loop
    pub fn setup_gl_resources(&self, gl: &glow::Context) -> Result<(), VisError> {
        unsafe {
            let vao = gl.create_vertex_array().unwrap();
            gl.bind_vertex_array(Some(vao));
            gl.enable(glow::DEPTH_TEST);
            gl.clear_color(0.0, 0.0, 0.0, 1.0);
        }
        self.globe.program.bind(gl);
        self.globe.buffer.bind(gl);
        set_attrib(gl, &self.globe.program, "position", 3, 3, 0)?;

        self.mvp.proj.apply(gl);
        self.mvp.view.apply(gl);
        self.mvp.model.apply(gl);
        Ok(())
    }
}

impl Drop for VisGl {
    fn drop(&self, gl: &glow::Context) {
        self.globe.drop(gl);
    }
}

#[cfg(target_arch = "wasm32")]
struct VisContext {
    pub gl: glow::Context,
    pub shader_version: String,
}

#[cfg(target_arch = "wasm32")]
impl VisContext {
    pub fn new(width: f64, height: f64) -> Result<Self, VisError> {
        use web::*;
        let shader_version = String::from("#version 300 es");
        let canvas = window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("canvas"))
            .and_then(|e| e.dyn_into::<HtmlCanvasElement>().ok())
            .ok_or(VisError::WebSys)?;
        canvas.set_width(width as u32);
        canvas.set_height(height as u32);
        let ctx = canvas
            .get_context("webgl2")
            .ok()
            .and_then(|o| o)
            .and_then(|e| e.dyn_into::<WebGl2RenderingContext>().ok())
            .ok_or(VisError::WebSys)?;
        let gl = glow::Context::from_webgl2_context(ctx);
        Ok(Self { gl, shader_version })
    }

    pub fn run(window: VisContext, mut vis: VisGl) -> Result<(), VisError> {
        vis.setup_gl_resources(&window.gl)?;
        let mut draw = VisGl::get_draw();
        draw(&window.gl, &mut vis);
        Ok(())
    }
}

// contains gl context, window, event loop
#[cfg(not(target_arch = "wasm32"))]
struct VisContext {
    pub gl: glow::Context,
    pub shader_version: String,
    pub ctx: glutin::ContextWrapper<glutin::PossiblyCurrent, glutin::window::Window>,
    pub event_loop: glutin::event_loop::EventLoop<()>,
}

#[cfg(not(target_arch = "wasm32"))]
impl VisContext {
    pub fn new(width: f64, height: f64) -> Result<Self, VisError> {
        use native::*;
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_inner_size(LogicalSize::new(width, height))
            .with_title("window");
        let ctx_builder = ContextBuilder::new()
            .with_multisampling(4)
            .build_windowed(window, &event_loop)?;
        let ctx;
        let gl;
        unsafe {
            ctx = ctx_builder.make_current().unwrap();
            gl = glow::Context::from_loader_function(|x| ctx.get_proc_address(x) as *const _);
        }
        let shader_version = String::from("#version 410");
        Ok(Self {
            gl,
            shader_version,
            ctx,
            event_loop,
        })
    }

    // window passed as argument since running event loop causes move
    // calls vis event handlers on event
    pub fn run(window: VisContext, mut vis: VisGl) -> Result<(), VisError> {
        use native::*;
        vis.setup_gl_resources(&window.gl)?;
        let mut draw = VisGl::get_draw();
        window.event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CursorMoved { position, .. } => {
                        vis.mouse_move(&window.gl, position.x, position.y);
                    }
                    WindowEvent::MouseWheel { delta, .. } => {
                        let ds = match delta {
                            MouseScrollDelta::PixelDelta(position) => position.y,
                            MouseScrollDelta::LineDelta(_, y) => y as f64,
                        };
                        vis.mouse_wheel(&window.gl, ds);
                    }
                    WindowEvent::MouseInput { button, state, .. } => {
                        let button = match button {
                            MouseButtonGlutin::Left => MouseButton::Left,
                            MouseButtonGlutin::Right => MouseButton::Right,
                            _ => MouseButton::Other,
                        };
                        let state = match state {
                            ElementState::Pressed => true,
                            ElementState::Released => false,
                        };
                        vis.mouse_input(&window.gl, button, state);
                    }
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    _ => (),
                },
                Event::LoopDestroyed => {
                    vis.drop(&window.gl);
                }
                Event::RedrawRequested(_) => {
                    draw(&window.gl, &mut vis);
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
    pub fn new_default(
        gl: &glow::Context,
        program: &Program,
        aspect: f32,
    ) -> Result<Self, MvpError> {
        let proj = UniformMatrix::new(
            gl,
            program,
            "projMatrix",
            Mat4::perspective_rh_gl(1.25, aspect, 0.1, 10.0),
        )?;
        let view = UniformMatrix::new(
            gl,
            program,
            "viewMatrix",
            Mat4::look_at_rh(Vec3::new(0.0, 0.0, 2.0), Vec3::ZERO, Vec3::Y),
        )?;
        let model = UniformMatrix::new(gl, program, "modelMatrix", Mat4::IDENTITY)?;
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
    Attrib(#[from] crate::gl_wrap::AttribError),
    #[cfg(not(target_arch = "wasm32"))]
    #[error("{0}")]
    CtxCreation(#[from] glutin::CreationError),
    #[cfg(target_arch = "wasm32")]
    #[error("Web sys error")]
    WebSys,
}

#[derive(Error, Debug)]
pub enum MvpError {
    #[error("{0}")]
    UniformMatrix(#[from] crate::gl_wrap::UniformMatrixError),
}
