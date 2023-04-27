use crate::gl_wrap::{Buffer, Drop, Program};
use glow::HasContext;

pub struct Points {
    pub program: Program,
    pub buffer: Buffer,
}

impl Points {
    pub fn new(gl: &glow::Context, shader_version: &str) -> Result<Self, PointsError> {
        let buffer = Buffer::new(gl, glow::DYNAMIC_DRAW)?;
        let program = Program::new_from_sources(
            gl,
            shader_version,
            include_str!("../shaders/vert.glsl"),
            include_str!("../shaders/frag.glsl"),
        )?;
        Ok(Self { program, buffer })
    }

    pub fn get_draw() -> impl FnMut(&glow::Context, &mut Points) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        move |gl: &glow::Context, points: &mut Points| {
            let data: Vec<f32> = (0..300).map(|_| rng.gen_range(1.0..2.0)).collect();
            points.buffer.set_data(gl, &data);
            unsafe {
                gl.draw_arrays(glow::POINTS, 0, (data.len() / 3) as i32);
            }
        }
    }
}

impl Drop for Points {
    fn drop(&self, gl: &glow::Context) {
        self.program.drop(gl);
        self.buffer.drop(gl);
    }
}

use thiserror::Error;
#[derive(Error, Debug)]
pub enum PointsError {
    #[error("{0}")]
    Program(#[from] crate::gl_wrap::ProgramError),
    #[error("{0}")]
    Buffer(#[from] crate::gl_wrap::BufferError),
}
