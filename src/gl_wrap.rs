extern crate glow;
extern crate glutin;
use glam::Mat4;
use glow::HasContext;
use std::fs;

// free resources
pub trait Drop {
    fn drop(&self, gl: &glow::Context);
}

// set gl state
pub trait Bind {
    fn bind(&self, gl: &glow::Context);
}

pub struct Shader {
    pub id: glow::Shader,
}

impl Shader {
    pub fn new(
        gl: &glow::Context,
        version: &str,
        source_file: &str,
        shader_type: u32,
    ) -> Result<Self, ShaderError> {
        // load and compile shader from text file
        let source = fs::read_to_string(source_file)?;
        let id;
        unsafe {
            id = gl.create_shader(shader_type)?;
            gl.shader_source(id, &format!("{}\n{}", version, source));
            gl.compile_shader(id);
        }

        // check if shader compiled successfully
        let success;
        unsafe {
            success = gl.get_shader_compile_status(id);
        }
        if success {
            Ok(Self { id })
        } else {
            let log;
            unsafe {
                log = gl.get_shader_info_log(id);
            }
            Err(ShaderError::Compilation(log))
        }
    }
}

impl Drop for Shader {
    fn drop(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_shader(self.id);
        }
    }
}

pub struct Program {
    pub id: glow::Program,
}

impl Program {
    pub fn new(
        gl: &glow::Context,
        vertex_shader: &Shader,
        fragment_shader: &Shader,
    ) -> Result<Self, ProgramError> {
        // link shaders into program
        let id;
        unsafe {
            id = gl.create_program()?;
            gl.attach_shader(id, vertex_shader.id);
            gl.attach_shader(id, fragment_shader.id);
            gl.link_program(id);
        }

        let success;
        unsafe {
            success = gl.get_program_link_status(id);
        }
        if success {
            Ok(Self { id })
        } else {
            let log;
            unsafe {
                log = gl.get_program_info_log(id);
            }
            Err(ProgramError::Linking(log))
        }
    }

    // constructor from files for convenience
    pub fn new_from_files(
        gl: &glow::Context,
        version: &str,
        vertex_file: &str,
        fragment_file: &str,
    ) -> Result<Self, ProgramError> {
        let vertex_shader = Shader::new(gl, version, vertex_file, gl::VERTEX_SHADER)?;
        let fragment_shader = Shader::new(gl, version, fragment_file, gl::FRAGMENT_SHADER)?;
        let result = Self::new(gl, &vertex_shader, &fragment_shader);

        // free no longer needed shader resources after linking
        vertex_shader.drop(gl);
        fragment_shader.drop(gl);

        // return result of default constructor
        result
    }
}

impl Drop for Program {
    fn drop(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_program(self.id);
        }
    }
}

impl Bind for Program {
    fn bind(&self, gl: &glow::Context) {
        unsafe {
            gl.use_program(Some(self.id));
        }
    }
}

pub struct Buffer {
    pub id: glow::Buffer,
    pub draw_type: u32,
}

impl Buffer {
    pub fn new(gl: &glow::Context, data: &[f32], draw_type: u32) -> Result<Self, BufferError> {
        let id;
        unsafe {
            id = gl.create_buffer()?;
        }
        let buffer = Self { id, draw_type };
        buffer.set_data(gl, data);
        Ok(buffer)
    }

    pub fn set_data(&self, gl: &glow::Context, data: &[f32]) {
        self.bind(gl);
        unsafe {
            let (_, bytes, _) = data.align_to::<u8>();
            gl.buffer_data_u8_slice(gl::ARRAY_BUFFER, bytes, self.draw_type);
        }
    }
}

impl Bind for Buffer {
    fn bind(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_buffer(gl::ARRAY_BUFFER, Some(self.id));
        }
    }
}

impl Drop for Buffer {
    fn drop(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_buffer(self.id);
        }
    }
}

pub struct UniformMatrix {
    pub data: Mat4,
    location: glow::UniformLocation,
}

impl UniformMatrix {
    pub fn new(
        gl: &glow::Context,
        program: &Program,
        name: &str,
        data: Mat4,
    ) -> Result<Self, UniformMatrixError> {
        let location;
        unsafe {
            location = gl.get_uniform_location(program.id, name);
        }
        match location {
            Some(location) => Ok(Self { location, data }),
            None => Err(UniformMatrixError::Location),
        }
    }

    pub fn apply(&self, gl: &glow::Context) {
        unsafe {
            gl.uniform_matrix_4_f32_slice(Some(&self.location), false, &self.data.to_cols_array());
        }
    }
}

pub fn set_attrib(
    gl: &glow::Context,
    program: &Program,
    name: &str,
    size: i32,
    stride: i32,
    offset: i32,
) -> Result<(), AttribError> {
    let location;
    unsafe {
        location = gl.get_attrib_location(program.id, name);
    }
    match location {
        None => Err(AttribError::Location),
        Some(location) => unsafe {
            let fsize = std::mem::size_of::<f32>() as i32;
            gl.vertex_attrib_pointer_f32(
                location,
                size,
                gl::FLOAT,
                false,
                fsize * stride,
                fsize * offset,
            );
            gl.enable_vertex_attrib_array(location);
            Ok(())
        },
    }
}

extern crate thiserror;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ShaderError {
    #[error("Compilation error: {0}")]
    Compilation(String),
    #[error("{0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Nul(#[from] std::ffi::NulError),
    #[error("{0}")]
    String(String),
}

impl From<String> for ShaderError {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

#[derive(Error, Debug)]
pub enum ProgramError {
    #[error("Linking error: {0}")]
    Linking(String),
    #[error("{0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("{0}")]
    Nul(#[from] std::ffi::NulError),
    #[error("{0}")]
    Shader(#[from] ShaderError),
    #[error("{0}")]
    String(String),
}

impl From<String> for ProgramError {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

#[derive(Error, Debug)]
pub enum BufferError {
    #[error("{0}")]
    String(String),
}

impl From<String> for BufferError {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

#[derive(Error, Debug)]
pub enum UniformMatrixError {
    #[error("{0}")]
    Nul(#[from] std::ffi::NulError),
    #[error("Uniform location not found")]
    Location,
}

#[derive(Error, Debug)]
pub enum AttribError {
    #[error("Attrib location not found")]
    Location,
}
