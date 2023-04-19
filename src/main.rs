extern crate gl;
extern crate glutin;
mod gl_wrap;
use gl_wrap::{Bind, Buffer, Drop, Program, VertexArray};
use glutin::dpi::LogicalSize;
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::{Api, ContextBuilder, GlRequest};

fn main() {
    // init gl window / ctx
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

    // init gl resources
    let program = Program::new_from_files("./shaders/vert.glsl", "./shaders/frag.glsl").unwrap();
    let data: [f32; 9] = [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0];
    let buffer = Buffer::new(&data, gl::STATIC_DRAW);
    let pos_loc = program.get_attrib_location("position").unwrap();
    let vertex_array = VertexArray::new();
    vertex_array.set_attribute(pos_loc, 3, 3, 0);
    program.bind();
    buffer.bind();
    vertex_array.bind();

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
                gl::DrawArrays(gl::TRIANGLES, 0, 3);
                ctx.swap_buffers().unwrap();
            },
            _ => (),
        }
    });
}
