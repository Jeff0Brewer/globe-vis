extern crate gl;
extern crate glutin;
use gl::types::GLenum;
use std::ffi::CString;
use std::{fs, ptr};

pub trait Drop {
    // free resources
    fn drop(&self);
}

pub trait Bind {
    // set gl state
    fn bind(&self);
}

pub struct Shader {
    pub id: u32,
}

impl Shader {
    pub fn new(source_file: &str, shader_type: GLenum) -> Result<Self, ShaderError> {
        // load and compile shader from text file
        let source_code = CString::new(fs::read_to_string(source_file)?)?;
        let shader: Self;
        unsafe {
            shader = Self {
                id: gl::CreateShader(shader_type),
            };
            gl::ShaderSource(shader.id, 1, &source_code.as_ptr(), ptr::null());
            gl::CompileShader(shader.id);
        }

        // check if shader compiled successfully
        let mut success: i32 = 0;
        unsafe {
            gl::GetShaderiv(shader.id, gl::COMPILE_STATUS, &mut success);
        }
        if success == 1 {
            Ok(shader)
        } else {
            // get shader info log and throw error if compilation failed
            let mut log_size: i32 = 0;
            unsafe {
                gl::GetShaderiv(shader.id, gl::INFO_LOG_LENGTH, &mut log_size);
            }
            let mut error_log: Vec<u8> = Vec::with_capacity(log_size as usize);
            unsafe {
                gl::GetShaderInfoLog(
                    shader.id,
                    log_size,
                    &mut log_size,
                    error_log.as_mut_ptr() as *mut _,
                );
                error_log.set_len(log_size as usize);
            }
            let log = String::from_utf8(error_log)?;
            Err(ShaderError::Compilation(log))
        }
    }
}

impl Drop for Shader {
    fn drop(&self) {
        unsafe { gl::DeleteShader(self.id) }
    }
}

pub struct Program {
    pub id: u32,
}

impl Program {
    pub fn new(vertex_shader: &Shader, fragment_shader: &Shader) -> Result<Self, ProgramError> {
        // link shaders into program
        let program: Self;
        unsafe {
            program = Self {
                id: gl::CreateProgram(),
            };
            gl::AttachShader(program.id, vertex_shader.id);
            gl::AttachShader(program.id, fragment_shader.id);
            gl::LinkProgram(program.id);
        }

        // check if program linked successfully
        let mut success: i32 = 0;
        unsafe {
            gl::GetProgramiv(program.id, gl::LINK_STATUS, &mut success);
        }
        if success == 1 {
            Ok(program)
        } else {
            // get program info log and throw error if linking failed
            let mut log_size: i32 = 0;
            unsafe {
                gl::GetProgramiv(program.id, gl::INFO_LOG_LENGTH, &mut log_size);
            }
            let mut error_log: Vec<u8> = Vec::with_capacity(log_size as usize);
            unsafe {
                gl::GetProgramInfoLog(
                    program.id,
                    log_size,
                    &mut log_size,
                    error_log.as_mut_ptr() as *mut _,
                );
                error_log.set_len(log_size as usize);
            }
            let log = String::from_utf8(error_log)?;
            Err(ProgramError::Linking(log))
        }
    }

    // constructor from files for convenience
    pub fn new_from_files(vertex_file: &str, fragment_file: &str) -> Result<Self, ProgramError> {
        let vertex_shader = Shader::new(vertex_file, gl::VERTEX_SHADER)?;
        let fragment_shader = Shader::new(fragment_file, gl::FRAGMENT_SHADER)?;
        let result = Self::new(&vertex_shader, &fragment_shader);

        // free no longer needed shader resources after linking
        vertex_shader.drop();
        fragment_shader.drop();

        // return result of default constructor
        result
    }

    pub fn set_attrib(
        &self,
        name: &str,
        size: i32,
        stride: i32,
        offset: i32,
    ) -> Result<(), ProgramError> {
        let name = CString::new(name)?;
        let fsize = std::mem::size_of::<f32>() as i32;
        unsafe {
            let location = gl::GetAttribLocation(self.id, name.as_ptr()) as u32;
            gl::VertexAttribPointer(
                location,
                size,
                gl::FLOAT,
                gl::FALSE,
                stride * fsize,
                (offset * fsize) as *const _,
            );
            gl::EnableVertexAttribArray(location);
        }
        Ok(())
    }
}

impl Drop for Program {
    fn drop(&self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

impl Bind for Program {
    fn bind(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }
}

pub struct Buffer {
    pub id: u32,
    pub draw_type: u32,
}

impl Buffer {
    pub fn new(data: &[f32], draw_type: u32) -> Self {
        let mut id: u32 = 0;
        unsafe {
            gl::GenBuffers(1, &mut id);
        }
        let buffer = Self { id, draw_type };
        buffer.set_data(data);
        buffer
    }

    pub fn set_data(&self, data: &[f32]) {
        self.bind();
        unsafe {
            let (_, bytes, _) = data.align_to::<u8>();
            gl::BufferData(
                gl::ARRAY_BUFFER,
                bytes.len() as isize,
                bytes.as_ptr() as *const _,
                self.draw_type,
            )
        }
    }
}

impl Bind for Buffer {
    fn bind(&self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.id);
        }
    }
}

impl Drop for Buffer {
    fn drop(&self) {
        unsafe {
            gl::DeleteBuffers(1, [self.id].as_ptr());
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
}

#[derive(Error, Debug)]
pub enum ProgramError {
    #[error("Linking error: {0}")]
    Linking(String),
    #[error{"{0}"}]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error{"{0}"}]
    Nul(#[from] std::ffi::NulError),
    #[error{"{0}"}]
    Shader(#[from] ShaderError),
}
