extern crate gl;
extern crate glutin;
mod gl_wrap;
mod icosphere;
mod mouse;
use gl_wrap::{set_attrib, Bind, Buffer, Drop, Program};
use glam::{Mat4, Vec3};
use glutin::dpi::{LogicalSize, PhysicalPosition};
use glutin::event::{ElementState, Event, MouseButton, MouseScrollDelta, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::{Window, WindowBuilder};
use glutin::{ContextBuilder, ContextWrapper, GlRequest, PossiblyCurrent};
use icosphere::get_icosphere;
use mouse::{rotate_from_mouse, zoom_from_scroll};
use std::ffi::CString;

struct UniformMatrix {
    pub location: i32,
    pub data: Mat4,
}

impl UniformMatrix {
    pub fn new(program: &Program, name: &str, data: Mat4) -> Self {
        let name = CString::new(name).unwrap();
        let location;
        unsafe {
            location = gl::GetUniformLocation(program.id, name.as_ptr());
        }
        Self { location, data }
    }

    pub fn apply(&self) {
        unsafe {
            gl::UniformMatrix4fv(self.location, 1, gl::FALSE, &self.data.to_cols_array()[0]);
        }
    }
}

struct Globe {
    pub ctx: ContextWrapper<PossiblyCurrent, Window>,
    pub event_loop: EventLoop<()>,
    pub program: Program,
    pub buffer: Buffer,
    pub data: Vec<f32>,
    pub view_mat: UniformMatrix,
    pub proj_mat: UniformMatrix,
    pub model_mat: UniformMatrix,
}

impl Globe {
    pub fn new(width: f64, height: f64) -> Self {
        let (ctx, event_loop) = Globe::get_ctx(width, height);
        let (program, buffer, data, proj_mat, view_mat, model_mat) =
            Globe::get_resources((width / height) as f32);
        Self {
            ctx,
            event_loop,
            program,
            buffer,
            data,
            view_mat,
            proj_mat,
            model_mat,
        }
    }

    fn get_ctx(
        width: f64,
        height: f64,
    ) -> (ContextWrapper<PossiblyCurrent, Window>, EventLoop<()>) {
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
        (ctx, event_loop)
    }

    fn get_resources(
        aspect: f32,
    ) -> (
        Program,
        Buffer,
        Vec<f32>,
        UniformMatrix,
        UniformMatrix,
        UniformMatrix,
    ) {
        let data = get_icosphere(4);
        let buffer = Buffer::new(&data, gl::DYNAMIC_DRAW);
        let program =
            Program::new_from_files("./shaders/vert.glsl", "./shaders/frag.glsl").unwrap();
        let proj_mat = UniformMatrix::new(
            &program,
            "projMatrix",
            Mat4::perspective_rh_gl(70.0 * std::f32::consts::PI / 180.0, aspect, 0.01, 10.0),
        );
        let view_mat = UniformMatrix::new(
            &program,
            "viewMatrix",
            Mat4::look_at_rh(Vec3::new(0.0, 0.0, 2.0), Vec3::ZERO, Vec3::Y),
        );
        let model_mat = UniformMatrix::new(&program, "modelMatrix", Mat4::IDENTITY);
        (program, buffer, data, proj_mat, view_mat, model_mat)
    }

    pub fn run(mut self) {
        let mut buf_change = 1.0;
        let mut buf_change_dir = 1.0;
        let mut drag_state = ElementState::Released;
        let mut mouse_pos = PhysicalPosition { x: 0.0, y: 0.0 };

        set_attrib(&self.program, "position", 3, 3, 0).unwrap();
        self.program.bind();
        self.buffer.bind();
        self.proj_mat.apply();
        self.view_mat.apply();
        self.model_mat.apply();

        self.event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CursorMoved { position, .. } => {
                        if let ElementState::Pressed = drag_state {
                            self.model_mat.data = rotate_from_mouse(
                                self.model_mat.data,
                                position.x - mouse_pos.x,
                                position.y - mouse_pos.y,
                            );
                            self.model_mat.apply();
                            self.ctx.window().request_redraw();
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
                        self.view_mat.data = zoom_from_scroll(self.view_mat.data, ds);
                        self.view_mat.apply();
                        self.ctx.window().request_redraw();
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
                    self.program.drop();
                    self.buffer.drop();
                }
                Event::RedrawRequested(_) => {
                    if !(0.5..1.0).contains(&buf_change) {
                        buf_change_dir = -buf_change_dir;
                    }
                    buf_change += buf_change_dir * 0.001;
                    let data: Vec<f32> = self.data.iter().map(|x| x * buf_change).collect();
                    self.buffer.set_data(&data);
                    unsafe {
                        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                        gl::DrawArrays(gl::TRIANGLES, 0, (data.len() / 3) as i32);
                    }
                    self.ctx.swap_buffers().unwrap();
                    self.ctx.window().request_redraw();
                }
                _ => (),
            }
        });
    }
}

fn main() {
    let globe = Globe::new(500.0, 500.0);
    globe.run();
}
