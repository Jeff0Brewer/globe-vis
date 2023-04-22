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

struct VisWindow {
    pub ctx: ContextWrapper<PossiblyCurrent, Window>,
    pub event_loop: EventLoop<()>,
}

impl VisWindow {
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

    pub fn run(window: VisWindow, mut vis: VisGl) {
        vis.setup();

        let mut draw = VisGl::get_draw();

        window.event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CursorMoved { position, .. } => {
                        vis.mouse_move(position);
                    }
                    WindowEvent::MouseWheel { delta, .. } => {
                        vis.mouse_wheel(delta);
                    }
                    WindowEvent::MouseInput { button, state, .. } => {
                        vis.mouse_input(button, state);
                    }
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    _ => (),
                },
                Event::LoopDestroyed => {
                    vis.drop();
                }
                Event::RedrawRequested(_) => {
                    draw(&mut vis);
                    window.ctx.swap_buffers().unwrap();
                    window.ctx.window().request_redraw();
                }
                _ => (),
            }
        });
    }
}

struct VisGl {
    pub globe: Globe,
    pub mvp: MvpMatrices,
    pub mouse: Mouse,
}

impl VisGl {
    pub fn new(width: f64, height: f64) -> Self {
        let globe = Globe::new();
        let mvp = MvpMatrices::new(
            &globe.program,
            70.0 * PI / 180.0,
            (width / height) as f32,
            Vec3::new(0.0, 0.0, 2.0),
        );
        let mouse = Mouse::new();
        Self { globe, mvp, mouse }
    }

    fn mouse_move(&mut self, position: PhysicalPosition<f64>) {
        if self.mouse.dragging {
            self.mvp.model.data = rotate_from_mouse(
                self.mvp.model.data,
                position.x - self.mouse.x,
                position.y - self.mouse.y,
            );
            self.mvp.model.apply();
        }
        self.mouse.x = position.x;
        self.mouse.y = position.y;
    }

    fn mouse_input(&mut self, button: MouseButton, state: ElementState) {
        if let MouseButton::Left = button {
            self.mouse.dragging = match state {
                ElementState::Pressed => true,
                ElementState::Released => false,
            }
        }
    }

    fn mouse_wheel(&mut self, delta: MouseScrollDelta) {
        let ds = match delta {
            MouseScrollDelta::PixelDelta(position) => position.y,
            MouseScrollDelta::LineDelta(_, y) => y as f64,
        };
        self.mvp.view.data = zoom_from_scroll(self.mvp.view.data, ds);
        self.mvp.view.apply();
    }

    pub fn get_draw() -> impl FnMut(&mut VisGl) {
        let mut globe_draw = Globe::get_draw();
        move |vis: &mut VisGl| {
            unsafe {
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            }
            globe_draw(&mut vis.globe)
        }
    }

    pub fn setup(&self) {
        self.globe.program.bind();
        self.globe.buffer.bind();
        set_attrib(&self.globe.program, "position", 3, 3, 0).unwrap();

        self.mvp.proj.apply();
        self.mvp.view.apply();
        self.mvp.model.apply();
    }
}

impl Drop for VisGl {
    fn drop(&self) {
        self.globe.drop();
    }
}

struct Vis {
    gl: VisGl,
    window: VisWindow,
}

impl Vis {
    pub fn new(width: f64, height: f64) -> Self {
        let window = VisWindow::new(width, height);
        let gl = VisGl::new(width, height);
        Self { gl, window }
    }

    pub fn start(vis: Vis) {
        VisWindow::run(vis.window, vis.gl);
    }
}

fn main() {
    let vis = Vis::new(500.0, 500.0);
    Vis::start(vis);
}
