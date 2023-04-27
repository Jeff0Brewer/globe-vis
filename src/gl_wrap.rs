use glam::Mat4;
use glow::HasContext;

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
        source: &str,
        shader_type: u32,
    ) -> Result<Self, ShaderError> {
        // load and compile shader from text file
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
    pub fn new_from_sources(
        gl: &glow::Context,
        version: &str,
        vertex_source: &str,
        fragment_source: &str,
    ) -> Result<Self, ProgramError> {
        let vertex_shader = Shader::new(gl, version, vertex_source, glow::VERTEX_SHADER)?;
        let fragment_shader = Shader::new(gl, version, fragment_source, glow::FRAGMENT_SHADER)?;
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
    pub len: usize,
}

impl Buffer {
    pub fn new(gl: &glow::Context, draw_type: u32) -> Result<Self, BufferError> {
        let id;
        unsafe {
            id = gl.create_buffer()?;
        }
        let len: usize = 0;
        let buffer = Self { id, draw_type, len };
        Ok(buffer)
    }

    pub fn set_data(&mut self, gl: &glow::Context, data: &[f32]) {
        self.bind(gl);
        self.len = data.len();
        unsafe {
            let (_, bytes, _) = data.align_to::<u8>();
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, bytes, self.draw_type);
        }
    }
}

impl Bind for Buffer {
    fn bind(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.id));
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

pub struct VertexArray {
    pub id: glow::VertexArray,
}

impl VertexArray {
    pub fn new(gl: &glow::Context) -> Result<Self, VertexArrayError> {
        let id;
        unsafe {
            id = gl.create_vertex_array()?;
        }
        Ok(Self { id })
    }

    pub fn set_attrib(
        gl: &glow::Context,
        program: &Program,
        name: &str,
        size: i32,
        stride: i32,
        offset: i32,
    ) -> Result<(), VertexArrayError> {
        let location;
        unsafe {
            location = gl.get_attrib_location(program.id, name);
        }
        match location {
            None => Err(VertexArrayError::Location),
            Some(location) => unsafe {
                let fsize = std::mem::size_of::<f32>() as i32;
                gl.vertex_attrib_pointer_f32(
                    location,
                    size,
                    glow::FLOAT,
                    false,
                    fsize * stride,
                    fsize * offset,
                );
                gl.enable_vertex_attrib_array(location);
                Ok(())
            },
        }
    }
}

impl Bind for VertexArray {
    fn bind(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_vertex_array(Some(self.id));
        }
    }
}

impl Drop for VertexArray {
    fn drop(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_vertex_array(self.id);
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
pub enum VertexArrayError {
    #[error("{0}")]
    String(String),
    #[error("Attrib location not found")]
    Location,
}

impl From<String> for VertexArrayError {
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
