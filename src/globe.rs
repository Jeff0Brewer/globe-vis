use crate::gl_wrap::{Bind, Buffer, Drop, Program, VertexArray};
use crate::icosphere::get_icosphere;
use glow::HasContext;

// contains gl resources / logic for drawing globe
pub struct Globe {
    pub data: Vec<f32>,
    pub program: Program,
    pub buffer: Buffer,
    pub vao: VertexArray,
}

impl Globe {
    pub fn new(gl: &glow::Context, shader_version: &str) -> Result<Self, GlobeError> {
        let data = get_icosphere(4);
        let mut buffer = Buffer::new(gl, glow::STATIC_DRAW)?;
        buffer.set_data(gl, &data);
        let program = Program::new_from_sources(
            gl,
            shader_version,
            include_str!("../shaders/globe-vert.glsl"),
            include_str!("../shaders/globe-frag.glsl"),
        )?;
        let vao = VertexArray::new(gl)?;
        Ok(Self {
            data,
            program,
            buffer,
            vao,
        })
    }

    // get draw function as closure for flexibility
    pub fn get_draw() -> impl FnMut(&glow::Context, &mut Globe) {
        |gl: &glow::Context, globe: &mut Globe| unsafe {
            gl.draw_arrays(glow::TRIANGLES, 0, (globe.buffer.len / 3) as i32);
        }
    }

    pub fn setup_gl_resources(&self, gl: &glow::Context) -> Result<(), GlobeError> {
        self.program.bind(gl);
        self.buffer.bind(gl);
        self.vao.bind(gl);
        VertexArray::set_attrib(gl, &self.program, "position", 3, 3, 0)?;
        Ok(())
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
    #[error("{0}")]
    VertexArray(#[from] crate::gl_wrap::VertexArrayError),
}
