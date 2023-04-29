use crate::{
    gl_wrap::{Drop, Program, UniformMatrix},
    globe::Globe,
    mouse::{rotate_from_mouse, zoom_from_scroll, MouseButtons, MouseState},
    points::Points,
    vis_ctx::{VisContext, VisContextError},
    VisState,
};
use glam::{Mat4, Vec3};
use glow::HasContext;

// wrapper for initialization and running vis
pub struct VisBuilder<T: VisState + 'static> {
    width: Option<f64>,
    height: Option<f64>,
    state: Option<T>,
}

impl<T: VisState + 'static> VisBuilder<T> {
    pub fn new() -> Self {
        let width = None;
        let height = None;
        let state = None;
        Self {
            width,
            height,
            state,
        }
    }

    // set window size
    pub fn with_dimensions(mut self, width: f64, height: f64) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    // add update fn
    pub fn with_state(mut self, state: T) -> Self {
        self.state = Some(state);
        self
    }

    // run visualization from prev set fields
    pub fn start(&mut self) -> Result<(), VisError> {
        let width = self.width.unwrap_or(500.0);
        let height = self.height.unwrap_or(500.0);
        let state = self.state.take();

        let window = VisContext::new(width, height)?;
        let gl = VisGl::new(&window, width, height)?;
        VisContext::run(window, gl, state)?;
        Ok(())
    }
}

// contains all vis logic and gl resources
pub struct VisGl {
    pub globe: Globe,
    pub points: Points,
    pub mvp: MvpMatrices,
    pub mouse: MouseState,
}

impl VisGl {
    pub fn new(context: &VisContext, width: f64, height: f64) -> Result<Self, VisGlError> {
        let mouse = MouseState::new();
        let globe = Globe::new(&context.gl, &context.shader_version)?;
        let points = Points::new(&context.gl, &context.shader_version)?;
        let mvp = MvpMatrices::new_default((width / height) as f32)?;
        Ok(Self {
            globe,
            points,
            mvp,
            mouse,
        })
    }

    pub fn mouse_move(&mut self, gl: &glow::Context, x: f64, y: f64) -> Result<(), VisGlError> {
        if self.mouse.dragging {
            let dx = x - self.mouse.x;
            let dy = y - self.mouse.y;
            // rotate model matrix from mouse move deltas
            self.mvp.model.data = rotate_from_mouse(self.mvp.model.data, dx, dy);
            self.mvp.model.apply(gl, &VisGl::programs(self))?;
        }
        // save last mouse position
        self.mouse.x = x;
        self.mouse.y = y;
        Ok(())
    }

    pub fn mouse_wheel(&mut self, gl: &glow::Context, delta: f64) -> Result<(), VisGlError> {
        self.mvp.view.data = zoom_from_scroll(self.mvp.view.data, delta);
        self.mvp.view.apply(gl, &VisGl::programs(self))?;
        Ok(())
    }

    pub fn mouse_input(&mut self, _: &glow::Context, button: MouseButtons, pressed: bool) {
        // save mouse drag state on left mouse input
        if let MouseButtons::Left = button {
            self.mouse.dragging = pressed;
        }
    }

    // get main draw loop as closure
    pub fn get_draw() -> impl FnMut(&glow::Context, &mut VisGl, Option<Vec<f32>>) {
        let mut globe_draw = Globe::get_draw();
        let mut points_draw = Points::get_draw();
        move |gl: &glow::Context, vis: &mut VisGl, data: Option<Vec<f32>>| {
            unsafe {
                gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
            }
            globe_draw(gl, &mut vis.globe);
            points_draw(gl, &mut vis.points, data);
        }
    }

    // bind required resources for start of draw loop
    pub fn setup_gl_resources(&self, gl: &glow::Context) -> Result<(), VisGlError> {
        unsafe {
            gl.enable(glow::DEPTH_TEST);
            gl.clear_color(0.0, 0.0, 0.0, 1.0);
        }
        self.globe.setup_gl_resources(gl)?;
        self.points.setup_gl_resources(gl)?;

        self.mvp.apply(gl, &VisGl::programs(self)).unwrap();
        Ok(())
    }

    fn programs(vis: &VisGl) -> Vec<&Program> {
        vec![&vis.points.program, &vis.globe.program]
    }
}

impl Drop for VisGl {
    fn drop(&self, gl: &glow::Context) {
        self.globe.drop(gl);
        self.points.drop(gl);
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
    pub fn new_default(aspect: f32) -> Result<Self, MvpError> {
        let proj = UniformMatrix::new(
            "projMatrix",
            Mat4::perspective_rh_gl(1.25, aspect, 0.1, 10.0),
        );
        let view = UniformMatrix::new(
            "viewMatrix",
            Mat4::look_at_rh(Vec3::new(0.0, 0.0, 2.0), Vec3::ZERO, Vec3::Y),
        );
        let model = UniformMatrix::new("modelMatrix", Mat4::IDENTITY);
        Ok(Self { proj, view, model })
    }

    pub fn apply(&self, gl: &glow::Context, programs: &[&Program]) -> Result<(), MvpError> {
        self.proj.apply(gl, programs)?;
        self.view.apply(gl, programs)?;
        self.model.apply(gl, programs)?;
        Ok(())
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
    Globe(#[from] crate::globe::GlobeError),
    #[error("{0}")]
    Points(#[from] crate::points::PointsError),
    #[error("{0}")]
    Mvp(#[from] MvpError),
    #[error("{0}")]
    UniformMatrix(#[from] crate::gl_wrap::UniformMatrixError),
}

#[derive(Error, Debug)]
pub enum MvpError {
    #[error("{0}")]
    UniformMatrix(#[from] crate::gl_wrap::UniformMatrixError),
}
