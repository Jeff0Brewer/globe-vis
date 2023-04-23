extern crate glow;
use crate::gl_wrap::{Buffer, Drop, Program};
use crate::icosphere::get_icosphere;
use glow::HasContext;

// contains gl resources / logic for drawing globe
pub struct Globe {
    pub data: Vec<f32>,
    pub program: Program,
    pub buffer: Buffer,
}

impl Globe {
    pub fn new(gl: &glow::Context, shader_version: &str) -> Result<Self, GlobeError> {
        let data = get_icosphere(4);
        let buffer = Buffer::new(gl, &data, glow::DYNAMIC_DRAW)?;
        let program = Program::new_from_files(
            gl,
            shader_version,
            "./shaders/vert.glsl",
            "./shaders/frag.glsl",
        )?;
        Ok(Self {
            program,
            buffer,
            data,
        })
    }

    // get draw function as closure for flexibility
    pub fn get_draw() -> impl FnMut(&glow::Context, &mut Globe) {
        // temp update to buffer for testing
        let mut buf_change = 1.0;
        let mut buf_change_dir = 1.0;
        move |gl: &glow::Context, globe: &mut Globe| {
            if !(0.5..1.0).contains(&buf_change) {
                buf_change_dir = -buf_change_dir;
            }
            buf_change += buf_change_dir * 0.001;
            let data: Vec<f32> = globe.data.iter().map(|x| x * buf_change).collect();
            globe.buffer.set_data(gl, &data);
            unsafe {
                gl.draw_arrays(glow::TRIANGLES, 0, (data.len() / 3) as i32);
            }
        }
    }
}

impl Drop for Globe {
    fn drop(&self, gl: &glow::Context) {
        self.program.drop(gl);
        self.buffer.drop(gl);
    }
}

use thiserror::Error;
#[derive(Error, Debug)]
pub enum GlobeError {
    #[error("{0}")]
    Program(#[from] crate::gl_wrap::ProgramError),
    #[error("{0}")]
    Buffer(#[from] crate::gl_wrap::BufferError),
}
