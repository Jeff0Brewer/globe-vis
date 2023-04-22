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
use std::f32::consts::PI;
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

impl MvpMatrices {
    pub fn new(program: &Program, fov: f32, aspect: f32, camera: Vec3) -> Self {
        Self {
            proj: UniformMatrix::new(
                program,
                "projMatrix",
                Mat4::perspective_rh_gl(fov, aspect, 0.1, 10.0),
            ),
            view: UniformMatrix::new(
                program,
                "viewMatrix",
                Mat4::look_at_rh(camera, Vec3::ZERO, Vec3::Y),
            ),
            model: UniformMatrix::new(program, "modelMatrix", Mat4::IDENTITY),
        }
    }
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

struct GlWindow {
    pub ctx: ContextWrapper<PossiblyCurrent, Window>,
    pub event_loop: EventLoop<()>,
}

impl GlWindow {
    pub fn new(width: f64, height: f64) -> Self {
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
        Self { ctx, event_loop }
    }
}

struct Globe {
    pub data: Vec<f32>,
    pub program: Program,
    pub buffer: Buffer,
}

impl Globe {
    pub fn new() -> Self {
        let data = get_icosphere(4);
        let buffer = Buffer::new(&data, gl::DYNAMIC_DRAW);
        let program =
            Program::new_from_files("./shaders/vert.glsl", "./shaders/frag.glsl").unwrap();
        Self {
            program,
            buffer,
            data,
        }
    }

    fn get_draw() -> impl FnMut(&mut Globe) {
        let mut buf_change = 1.0;
        let mut buf_change_dir = 1.0;
        move |globe: &mut Globe| {
            if !(0.5..1.0).contains(&buf_change) {
                buf_change_dir = -buf_change_dir;
            }
            buf_change += buf_change_dir * 0.001;
            let data: Vec<f32> = globe.data.iter().map(|x| x * buf_change).collect();
            globe.buffer.set_data(&data);
            unsafe {
                gl::DrawArrays(gl::TRIANGLES, 0, (data.len() / 3) as i32);
            }
        }
    }
}

impl Drop for Globe {
    fn drop(&self) {
        self.program.drop();
        self.buffer.drop();
    }
}

struct Vis {
    pub gl_window: GlWindow,
    pub globe: Globe,
    pub mvp: MvpMatrices,
    pub mouse: Mouse,
}

impl Vis {
    pub fn new(width: f64, height: f64) -> Self {
        let gl_window = GlWindow::new(width, height);
        let globe = Globe::new();
        let mvp = MvpMatrices::new(
            &globe.program,
            70.0 * PI / 180.0,
            (width / height) as f32,
            Vec3::new(0.0, 0.0, 2.0),
        );
        let mouse = Mouse::new();
        Self {
            gl_window,
            globe,
            mvp,
            mouse,
        }
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

    pub fn run(mut self) {
        self.globe.program.bind();
        self.globe.buffer.bind();
        set_attrib(&self.globe.program, "position", 3, 3, 0).unwrap();

        self.mvp.proj.apply();
        self.mvp.view.apply();
        self.mvp.model.apply();

        let mut globe_draw = Globe::get_draw();

        self.gl_window
            .event_loop
            .run(move |event, _, control_flow| {
                *control_flow = ControlFlow::Wait;
                match event {
                    Event::WindowEvent { event, .. } => match event {
                        WindowEvent::CursorMoved { position, .. } => {
                            Vis::mouse_move(&mut self.mouse, &mut self.mvp.model, position);
                        }
                        WindowEvent::MouseWheel { delta, .. } => {
                            Vis::mouse_wheel(&mut self.mvp.view, delta);
                        }
                        WindowEvent::MouseInput { button, state, .. } => {
                            Vis::mouse_input(&mut self.mouse, button, state);
                        }
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        _ => (),
                    },
                    Event::LoopDestroyed => {
                        self.globe.drop();
                    }
                    Event::RedrawRequested(_) => {
                        unsafe {
                            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                        }
                        globe_draw(&mut self.globe);
                        self.gl_window.ctx.swap_buffers().unwrap();
                        self.gl_window.ctx.window().request_redraw();
                    }
                    _ => (),
                }
            });
    }
}

fn main() {
    let globe = Vis::new(500.0, 500.0);
    globe.run();
}
