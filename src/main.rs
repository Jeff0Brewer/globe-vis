extern crate gl;
extern crate glutin;
use gl::types::{GLenum, GLint, GLuint};
use glutin::dpi::LogicalSize;
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::{Api, ContextBuilder, GlRequest};
use std::ffi::CString;
use std::{fs, ptr};

fn main() {
    // init gl resources
    let window = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(500.0, 500.0))
        .with_title("window");
    let event_loop = EventLoop::new();
    let ctx_builder = ContextBuilder::new()
        .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
        .with_multisampling(4)
        .build_windowed(window, &event_loop);
    let ctx;
    unsafe {
        ctx = ctx_builder.unwrap().make_current().unwrap();
        gl::load_with(|ptr| ctx.get_proc_address(ptr) as *const _);
    }
    let program = Program::new_from_files("./shaders/vert.glsl", "./shaders/frag.glsl").unwrap();
    let data: [f32; 9] = [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0];
    let buffer = Buffer::new(&data, gl::STATIC_DRAW);
    program.bind();
    buffer.bind();

    let pos_loc = program.get_attrib_location("position").unwrap();
    let float_size = core::mem::size_of::<f32>() as i32;
    let off: i32 = 0;
    unsafe {
        let mut id: GLuint = 0;
        gl::GenVertexArrays(1, &mut id);
        gl::BindVertexArray(id);
        gl::VertexAttribPointer(
            pos_loc,
            3,
            gl::FLOAT,
            gl::FALSE,
            3 * float_size,
            (off * float_size) as *const gl::types::GLvoid,
        );
        gl::EnableVertexAttribArray(pos_loc);
    }

    // begin draw loop
    ctx.swap_buffers().unwrap();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                _ => (),
            },
            Event::LoopDestroyed => {
                program.drop();
                buffer.drop();
            }
            Event::RedrawRequested(_) => unsafe {
                gl::Clear(gl::COLOR_BUFFER_BIT);
                gl::DrawArrays(gl::TRIANGLES, 0, 3);
                ctx.swap_buffers().unwrap();
            },
            _ => (),
        }
    });
}

pub trait Drop {
    fn drop(&self);
}

pub trait Bind {
    fn bind(&self);
}

pub struct Shader {
    pub id: GLuint,
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
        let mut success: GLint = 0;
        unsafe {
            gl::GetShaderiv(shader.id, gl::COMPILE_STATUS, &mut success);
        }
        if success == 1 {
            Ok(shader)
        } else {
            // get shader info log and throw error on compilation failure
            let mut log_size: GLint = 0;
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
    pub id: GLuint,
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
        let mut success: GLint = 0;
        unsafe {
            gl::GetProgramiv(program.id, gl::LINK_STATUS, &mut success);
        }
        if success == 1 {
            Ok(program)
        } else {
            // get program info log and throw error on linking failure
            let mut log_size: GLint = 0;
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

    // constructor from files to minimize initialization steps
    pub fn new_from_files(vertex_file: &str, fragment_file: &str) -> Result<Self, ProgramError> {
        let vertex_shader = Shader::new(vertex_file, gl::VERTEX_SHADER)?;
        let fragment_shader = Shader::new(fragment_file, gl::FRAGMENT_SHADER)?;
        let result = Self::new(&vertex_shader, &fragment_shader);

        // free unneccesary shader resources after linking
        vertex_shader.drop();
        fragment_shader.drop();

        // return result of default constructor
        result
    }

    pub fn get_attrib_location(&self, attrib: &str) -> Result<GLuint, ProgramError> {
        let attrib = CString::new(attrib)?;
        unsafe { Ok(gl::GetAttribLocation(self.id, attrib.as_ptr()) as GLuint) }
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
    pub id: GLuint,
}

impl Buffer {
    fn new(data: &[f32], draw_type: GLuint) -> Self {
        let mut id: GLuint = 0;
        unsafe {
            gl::GenBuffers(1, &mut id);
        }
        let buffer = Self { id };
        buffer.set_data(data, draw_type);
        buffer
    }

    pub fn set_data(&self, data: &[f32], draw_type: GLuint) {
        self.bind();
        unsafe {
            let (_, bytes, _) = data.align_to::<u8>();
            gl::BufferData(
                gl::ARRAY_BUFFER,
                bytes.len() as isize,
                bytes.as_ptr() as *const _,
                draw_type,
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
