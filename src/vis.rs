use crate::gl_wrap::{set_attrib, Bind, Drop, Program, UniformMatrix};
use crate::globe::Globe;
use crate::mouse::{rotate_from_mouse, zoom_from_scroll, MouseButton, MouseState};
use crate::vis_ctx::{VisContext, VisContextError};
use glam::{Mat4, Vec3};
use glow::HasContext;

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
pub struct VisGl {
    pub globe: Globe,
    pub mvp: MvpMatrices,
    pub mouse: MouseState,
}

impl VisGl {
    pub fn new(context: &VisContext, width: f64, height: f64) -> Result<Self, VisGlError> {
        let mouse = MouseState::new();
        let globe = Globe::new(&context.gl, &context.shader_version)?;
        let mvp = MvpMatrices::new_default(&context.gl, &globe.program, (width / height) as f32)?;
        Ok(Self { globe, mvp, mouse })
    }

    pub fn mouse_move(&mut self, gl: &glow::Context, x: f64, y: f64) {
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

    pub fn mouse_input(&mut self, _: &glow::Context, button: MouseButton, pressed: bool) {
        // save mouse drag state on left mouse input
        if let MouseButton::Left = button {
            self.mouse.dragging = pressed;
        }
    }

    pub fn mouse_wheel(&mut self, gl: &glow::Context, delta: f64) {
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
    pub fn setup_gl_resources(&self, gl: &glow::Context) -> Result<(), VisGlError> {
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

// matrices for 3D scene
// one instance for all programs, same matrices used everywhere
pub struct MvpMatrices {
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
    VisGl(#[from] VisGlError),
    #[error("{0}")]
    VisContext(#[from] VisContextError),
}

#[derive(Error, Debug)]
pub enum VisGlError {
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
    #[error("Web sys canvas retrieval failed")]
    Canvas,
    #[cfg(target_arch = "wasm32")]
    #[error("Web sys webgl2 context creation failed")]
    WebGl2Context,
}

#[derive(Error, Debug)]
pub enum MvpError {
    #[error("{0}")]
    UniformMatrix(#[from] crate::gl_wrap::UniformMatrixError),
}
