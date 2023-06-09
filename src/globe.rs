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
        // compile program from strings
        let program = Program::new_from_sources(
            gl,
            shader_version,
            include_str!("../shaders/globe-vert.glsl"),
            include_str!("../shaders/globe-frag.glsl"),
        )?;
        // init buffer and set data
        let data = get_icosphere(4);
        let mut buffer = Buffer::new(gl, glow::STATIC_DRAW)?;
        buffer.set_data(gl, &data);
        // init vao and setup attributes
        let vao = VertexArray::new(gl)?;
        program.bind(gl);
        buffer.bind(gl);
        vao.bind(gl);
        VertexArray::set_attrib(gl, &program, "position", 3, 3, 0)?;
        Ok(Self {
            data,
            program,
            buffer,
            vao,
        })
    }

    pub fn draw(&self, gl: &glow::Context) {
        self.program.bind(gl);
        self.buffer.bind(gl);
        self.vao.bind(gl);
        unsafe {
            gl.draw_arrays(glow::TRIANGLES, 0, (self.buffer.len / 3) as i32);
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
    #[error("{0}")]
    VertexArray(#[from] crate::gl_wrap::VertexArrayError),
}
