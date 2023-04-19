extern crate gl;
extern crate glutin;
mod gl_wrap;
mod icosphere;
use gl_wrap::{Bind, Buffer, Drop, Program, VertexArray};
use glam::{Mat4, Vec3};
use glutin::dpi::LogicalSize;
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::{Api, ContextBuilder, GlRequest};
use icosphere::get_icosphere;
use std::ffi::CString;

fn main() {
    let width = 500.0;
    let height = 500.0;
    // init gl window / ctx
    let window = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(width, height))
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

    // init gl resources
    let program = Program::new_from_files("./shaders/vert.glsl", "./shaders/frag.glsl").unwrap();
    let data = get_icosphere(4);
    let buffer = Buffer::new(&data, gl::STATIC_DRAW);
    let pos_loc = program.get_attrib_location("position").unwrap();
    let vertex_array = VertexArray::new();
    vertex_array.set_attribute(pos_loc, 3, 3, 0);
    program.bind();
    buffer.bind();
    vertex_array.bind();

    // init mvp uniform
    let view_mat = Mat4::look_at_rh(Vec3::new(0.0, 0.0, 2.0), Vec3::ZERO, Vec3::Y);
    let proj_mat = Mat4::perspective_rh_gl(
        70.0 * std::f32::consts::PI / 180.0,
        width / height,
        0.01,
        10.0,
    );
    let mvp = proj_mat.mul_mat4(&view_mat).to_cols_array();
    let cname = CString::new("mvp").unwrap();
    unsafe {
        let location = gl::GetUniformLocation(program.id, cname.as_ptr());
        gl::UniformMatrix4fv(location, 1, gl::FALSE, &mvp[0]);
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
                vertex_array.drop();
            }
            Event::RedrawRequested(_) => unsafe {
                gl::Clear(gl::COLOR_BUFFER_BIT);
                gl::DrawArrays(gl::TRIANGLES, 0, (data.len() / 3) as i32);
                ctx.swap_buffers().unwrap();
            },
            _ => (),
        }
    });
}
