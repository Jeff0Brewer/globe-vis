use crate::{
    gl_wrap::{Drop, Program, UniformMatrix},
    globe::Globe,
    mouse::{rotate_from_mouse, zoom_from_scroll, MouseButtons, MouseState},
    points::Points,
    vis_ctx::VisContext,
};
use glam::{Mat4, Vec3};
use glow::HasContext;

// contains all vis gl resources and camera mouse handlers
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

    // set gl features and required values for start of draw loop
    pub fn setup_gl(&self, gl: &glow::Context) -> Result<(), VisGlError> {
        unsafe {
            gl.clear_color(0.0, 0.0, 0.0, 1.0);
            gl.enable(glow::DEPTH_TEST);
            // point size feature not needed for wasm
            #[cfg(not(target_arch = "wasm32"))]
            gl.enable(glow::PROGRAM_POINT_SIZE);
        }
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
