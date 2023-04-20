extern crate gl;
extern crate glutin;
mod gl_wrap;
mod icosphere;
use gl_wrap::{Bind, Buffer, Drop, Program};
use glam::{Mat4, Vec3};
use glutin::dpi::{LogicalSize, PhysicalPosition};
use glutin::event::{ElementState, Event, MouseButton, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::{ContextBuilder, GlRequest};
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
        .with_gl(GlRequest::GlThenGles {
            // version 2.0 for WebGL compatibility
            opengl_version: (2, 0),
            opengles_version: (2, 0),
        })
        .with_multisampling(4)
        .build_windowed(window, &event_loop);
    let ctx;
    unsafe {
        ctx = ctx_builder.unwrap().make_current().unwrap();
        gl::load_with(|ptr| ctx.get_proc_address(ptr) as *const _);
        gl::Enable(gl::DEPTH_TEST);
    }

    // init gl resources
    let data = get_icosphere(4);
    let buffer = Buffer::new(&data, gl::STATIC_DRAW);
    let program = Program::new_from_files("./shaders/vert.glsl", "./shaders/frag.glsl").unwrap();
    program.set_attrib("position", 3, 3, 0).unwrap();
    program.bind();
    buffer.bind();

    // init mvp uniform
    let view_mat = Mat4::look_at_rh(Vec3::new(0.0, 0.0, 2.0), Vec3::ZERO, Vec3::Y);
    let proj_mat = Mat4::perspective_rh_gl(
        70.0 * std::f32::consts::PI / 180.0,
        width / height,
        0.01,
        10.0,
    );
    let mut mvp = proj_mat.mul_mat4(&view_mat);
    let cname = CString::new("mvp").unwrap();
    let mvp_loc;
    unsafe {
        mvp_loc = gl::GetUniformLocation(program.id, cname.as_ptr());
    }

    // begin draw loop
    let mut drag_state = ElementState::Released;
    let mut mouse_pos = PhysicalPosition { x: 0.0, y: 0.0 };
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CursorMoved { position, .. } => {
                    if let ElementState::Pressed = drag_state {
                        let dx = position.x - mouse_pos.x;
                        let dy = position.y - mouse_pos.y;
                        let rotation = rotation_from_mouse(dx, dy);
                        mvp = mvp.mul_mat4(&rotation);
                        ctx.window().request_redraw();
                    }
                    mouse_pos = PhysicalPosition {
                        x: position.x,
                        y: position.y,
                    };
                }
                WindowEvent::MouseInput {
                    button: MouseButton::Left,
                    state,
                    ..
                } => drag_state = state,
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                _ => (),
            },
            Event::LoopDestroyed => {
                program.drop();
                buffer.drop();
            }
            Event::RedrawRequested(_) => unsafe {
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                gl::UniformMatrix4fv(mvp_loc, 1, gl::FALSE, &mvp.to_cols_array()[0]);
                gl::DrawArrays(gl::TRIANGLES, 0, (data.len() / 3) as i32);
                ctx.swap_buffers().unwrap();
            },
            _ => (),
        }
    });
}

fn rotation_from_mouse(dx: f64, dy: f64) -> Mat4 {
    let rotation_speed = 0.1;
    let x_rad = (-dy * rotation_speed) as f32;
    let x_rotation = Mat4::from_rotation_x(x_rad);
    let z_rad = (dx * rotation_speed) as f32;
    let z_rotation = Mat4::from_rotation_z(z_rad);
    x_rotation.mul_mat4(&z_rotation)
}
