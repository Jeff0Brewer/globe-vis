use crate::gl_wrap::{Bind, Buffer, Drop, Program, VertexArray};
use glow::HasContext;

pub struct Points {
    pub program: Program,
    pub buffer: Buffer,
    pub vao: VertexArray,
}

impl Points {
    pub fn new(gl: &glow::Context, shader_version: &str) -> Result<Self, PointsError> {
        // compile program from strings
        let program = Program::new_from_sources(
            gl,
            shader_version,
            include_str!("../shaders/point-vert.glsl"),
            include_str!("../shaders/point-frag.glsl"),
        )?;
        // init empty buffer
        let buffer = Buffer::new(gl, glow::DYNAMIC_DRAW)?;
        // init vao and setup attributes
        let vao = VertexArray::new(gl)?;
        program.bind(gl);
        buffer.bind(gl);
        vao.bind(gl);
        VertexArray::set_attrib(gl, &program, "position", 3, 3, 0)?;
        Ok(Self {
            program,
            buffer,
            vao,
        })
    }

    pub fn draw(&mut self, gl: &glow::Context, data: Option<Vec<f32>>) {
        self.program.bind(gl);
        self.buffer.bind(gl);
        self.vao.bind(gl);
        if let Some(d) = data {
            self.buffer.set_data(gl, &d);
        }
        unsafe {
            gl.draw_arrays(glow::POINTS, 0, (self.buffer.len / 3) as i32);
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
    #[error("{0}")]
    VertexArray(#[from] crate::gl_wrap::VertexArrayError),
}
