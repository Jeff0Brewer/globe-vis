extern crate gl;
extern crate glutin;
mod gl_wrap;
mod icosphere;
mod mouse;
use gl_wrap::{Bind, Buffer, Drop, Program};
use glam::{Mat4, Vec3};
use glutin::dpi::{LogicalSize, PhysicalPosition};
use glutin::event::{ElementState, Event, MouseButton, MouseScrollDelta, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::{ContextBuilder, GlRequest};
use icosphere::get_icosphere;
use mouse::{rotate_from_mouse, zoom_from_scroll};
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

    // init matrix uniforms
    let proj_name = CString::new("projMatrix").unwrap();
    let proj_mat = Mat4::perspective_rh_gl(
        70.0 * std::f32::consts::PI / 180.0,
        width / height,
        0.01,
        10.0,
    );

    let view_name = CString::new("viewMatrix").unwrap();
    let mut view_mat = Mat4::look_at_rh(Vec3::new(0.0, 0.0, 2.0), Vec3::ZERO, Vec3::Y);
    let view_loc;

    let model_name = CString::new("modelMatrix").unwrap();
    let mut model_mat = Mat4::IDENTITY;
    let model_loc;

    unsafe {
        let proj_loc = gl::GetUniformLocation(program.id, proj_name.as_ptr());
        gl::UniformMatrix4fv(proj_loc, 1, gl::FALSE, &proj_mat.to_cols_array()[0]);

        view_loc = gl::GetUniformLocation(program.id, view_name.as_ptr());
        gl::UniformMatrix4fv(view_loc, 1, gl::FALSE, &view_mat.to_cols_array()[0]);

        model_loc = gl::GetUniformLocation(program.id, model_name.as_ptr());
        gl::UniformMatrix4fv(model_loc, 1, gl::FALSE, &model_mat.to_cols_array()[0]);
    }

    let mut buf_change = 1.0;
    let mut buf_change_dir = 1.0;

    // begin draw loop
    let mut drag_state = ElementState::Released;
    let mut mouse_pos = PhysicalPosition { x: 0.0, y: 0.0 };
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CursorMoved { position, .. } => {
                    if let ElementState::Pressed = drag_state {
                        model_mat = rotate_from_mouse(
                            model_mat,
                            position.x - mouse_pos.x,
                            position.y - mouse_pos.y,
                        );
                        unsafe {
                            gl::UniformMatrix4fv(
                                model_loc,
                                1,
                                gl::FALSE,
                                &model_mat.to_cols_array()[0],
                            );
                        }
                        ctx.window().request_redraw();
                    }
                    mouse_pos = PhysicalPosition {
                        x: position.x,
                        y: position.y,
                    };
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    let ds = match delta {
                        MouseScrollDelta::PixelDelta(position) => position.y,
                        MouseScrollDelta::LineDelta(_, y) => y as f64,
                    };
                    view_mat = zoom_from_scroll(view_mat, ds);
                    unsafe {
                        gl::UniformMatrix4fv(view_loc, 1, gl::FALSE, &view_mat.to_cols_array()[0]);
                    }
                    ctx.window().request_redraw();
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
            Event::RedrawRequested(_) => {
                if !(0.5..1.0).contains(&buf_change) {
                    buf_change_dir = -buf_change_dir;
                }
                buf_change += buf_change_dir * 0.001;
                let data: Vec<f32> = data.iter().map(|x| x * buf_change).collect();
                buffer.set_data(&data, gl::DYNAMIC_DRAW);
                unsafe {
                    gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                    gl::DrawArrays(gl::TRIANGLES, 0, (data.len() / 3) as i32);
                }
                ctx.swap_buffers().unwrap();
                ctx.window().request_redraw();
            }
            _ => (),
        }
    });
}
