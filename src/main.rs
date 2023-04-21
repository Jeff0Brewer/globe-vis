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

struct MvpMatrices {
    pub proj: UniformMatrix,
    pub view: UniformMatrix,
    pub model: UniformMatrix,
}

struct Mouse {
    x: f64,
    y: f64,
    dragging: bool,
}

impl Mouse {
    pub fn new() -> Self {
        Mouse {
            x: 0.0,
            y: 0.0,
            dragging: false,
        }
    }
}

struct Globe {
    pub ctx: ContextWrapper<PossiblyCurrent, Window>,
    pub event_loop: EventLoop<()>,
    pub data: Vec<f32>,
    pub program: Program,
    pub buffer: Buffer,
    pub mvp: MvpMatrices,
    pub mouse: Mouse,
}

impl Globe {
    pub fn new(width: f64, height: f64) -> Self {
        let (ctx, event_loop) = Globe::init_ctx(width, height);
        let (program, buffer, data, mvp) = Globe::init_resources((width / height) as f32);
        let mouse = Mouse::new();
        Self {
            ctx,
            event_loop,
            program,
            buffer,
            data,
            mvp,
            mouse,
        }
    }

    fn init_ctx(
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

    fn init_resources(aspect: f32) -> (Program, Buffer, Vec<f32>, MvpMatrices) {
        let data = get_icosphere(4);
        let buffer = Buffer::new(&data, gl::DYNAMIC_DRAW);
        let program =
            Program::new_from_files("./shaders/vert.glsl", "./shaders/frag.glsl").unwrap();
        let mvp = MvpMatrices {
            proj: UniformMatrix::new(
                &program,
                "projMatrix",
                Mat4::perspective_rh_gl(70.0 * std::f32::consts::PI / 180.0, aspect, 0.01, 10.0),
            ),
            view: UniformMatrix::new(
                &program,
                "viewMatrix",
                Mat4::look_at_rh(Vec3::new(0.0, 0.0, 2.0), Vec3::ZERO, Vec3::Y),
            ),
            model: UniformMatrix::new(&program, "modelMatrix", Mat4::IDENTITY),
        };
        (program, buffer, data, mvp)
    }

    fn mouse_move(mouse: &mut Mouse, matrix: &mut UniformMatrix, position: PhysicalPosition<f64>) {
        if mouse.dragging {
            matrix.data =
                rotate_from_mouse(matrix.data, position.x - mouse.x, position.y - mouse.y);
            matrix.apply();
        }
        mouse.x = position.x;
        mouse.y = position.y;
    }

    fn mouse_input(mouse: &mut Mouse, button: MouseButton, state: ElementState) {
        if let MouseButton::Left = button {
            mouse.dragging = match state {
                ElementState::Pressed => true,
                ElementState::Released => false,
            }
        }
    }

    fn mouse_wheel(matrix: &mut UniformMatrix, delta: MouseScrollDelta) {
        let ds = match delta {
            MouseScrollDelta::PixelDelta(position) => position.y,
            MouseScrollDelta::LineDelta(_, y) => y as f64,
        };
        matrix.data = zoom_from_scroll(matrix.data, ds);
        matrix.apply();
    }

    fn get_draw() -> impl FnMut(&mut Vec<f32>, &mut Buffer) {
        let mut buf_change = 1.0;
        let mut buf_change_dir = 1.0;
        move |data: &mut Vec<f32>, buffer: &mut Buffer| {
            if !(0.5..1.0).contains(&buf_change) {
                buf_change_dir = -buf_change_dir;
            }
            buf_change += buf_change_dir * 0.001;
            let data: Vec<f32> = data.iter().map(|x| x * buf_change).collect();
            buffer.set_data(&data);
            unsafe {
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                gl::DrawArrays(gl::TRIANGLES, 0, (data.len() / 3) as i32);
            }
        }
    }

    pub fn run(mut self) {
        self.program.bind();
        self.buffer.bind();
        set_attrib(&self.program, "position", 3, 3, 0).unwrap();

        self.mvp.proj.apply();
        self.mvp.view.apply();
        self.mvp.model.apply();

        let mut draw = Globe::get_draw();

        self.event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CursorMoved { position, .. } => {
                        Globe::mouse_move(&mut self.mouse, &mut self.mvp.model, position);
                    }
                    WindowEvent::MouseWheel { delta, .. } => {
                        Globe::mouse_wheel(&mut self.mvp.view, delta);
                    }
                    WindowEvent::MouseInput { button, state, .. } => {
                        Globe::mouse_input(&mut self.mouse, button, state);
                    }
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    _ => (),
                },
                Event::LoopDestroyed => {
                    self.program.drop();
                    self.buffer.drop();
                }
                Event::RedrawRequested(_) => {
                    draw(&mut self.data, &mut self.buffer);
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
