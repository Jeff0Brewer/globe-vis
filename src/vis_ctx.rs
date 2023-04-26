use crate::gl_wrap::Drop;
use crate::mouse::MouseButton;
use crate::vis::{VisGl, VisGlError};

#[cfg(not(target_arch = "wasm32"))]
mod native {
    pub use glutin::dpi::LogicalSize;
    pub use glutin::event::MouseButton as MouseButtonGlutin;
    pub use glutin::event::{ElementState, Event, MouseScrollDelta, WindowEvent};
    pub use glutin::event_loop::ControlFlow;
    pub use glutin::event_loop::EventLoop;
    pub use glutin::window::WindowBuilder;
    pub use glutin::ContextBuilder;
}

#[cfg(target_arch = "wasm32")]
mod web {
    pub use wasm_bindgen::JsCast;
    pub use web_sys::{window, HtmlCanvasElement, WebGl2RenderingContext};
    pub use winit::{
        event::MouseButton as MouseButtonWinit,
        event::{ElementState, Event, MouseScrollDelta, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        platform::web::WindowExtWebSys,
        window::WindowBuilder,
    };
}

#[cfg(target_arch = "wasm32")]
pub struct VisContext {
    pub gl: glow::Context,
    pub shader_version: String,
    pub window: winit::window::Window,
    pub event_loop: winit::event_loop::EventLoop<()>,
}

#[cfg(target_arch = "wasm32")]
impl VisContext {
    pub fn new(width: f64, height: f64) -> Result<Self, VisContextError> {
        use web::*;
        let shader_version = String::from("#version 300 es");
        let event_loop = EventLoop::new();
        let winit_window = WindowBuilder::new()
            .with_title("window")
            .build(&event_loop)
            .unwrap();
        let canvas = winit_window.canvas();
        let window = window().unwrap();
        let document = window.document().unwrap();
        let body = document.body().unwrap();

        canvas
            .style()
            .set_css_text(&format!("width: {:.0}px; height: {:.0}px;", width, height));
        canvas.set_width(width as u32);
        canvas.set_height(height as u32);
        body.append_child(&canvas).unwrap();

        let ctx = canvas
            .get_context("webgl2")
            .ok()
            .and_then(|o| o)
            .and_then(|e| e.dyn_into::<WebGl2RenderingContext>().ok())
            .ok_or(VisContextError::WebGl2Context)?;
        let gl = glow::Context::from_webgl2_context(ctx);
        Ok(Self {
            gl,
            shader_version,
            window: winit_window,
            event_loop,
        })
    }

    pub fn run(context: VisContext, mut vis: VisGl) -> Result<(), VisContextError> {
        use web::*;
        vis.setup_gl_resources(&context.gl)?;
        let mut draw = VisGl::get_draw();

        context.event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CursorMoved { position, .. } => {
                        vis.mouse_move(&context.gl, position.x, position.y);
                    }
                    WindowEvent::MouseWheel { delta, .. } => {
                        let ds = match delta {
                            MouseScrollDelta::PixelDelta(position) => position.y,
                            MouseScrollDelta::LineDelta(_, y) => y as f64,
                        };
                        vis.mouse_wheel(&context.gl, ds);
                    }
                    WindowEvent::MouseInput { button, state, .. } => {
                        let button = match button {
                            MouseButtonWinit::Left => MouseButton::Left,
                            MouseButtonWinit::Right => MouseButton::Right,
                            _ => MouseButton::Other,
                        };
                        let state = match state {
                            ElementState::Pressed => true,
                            ElementState::Released => false,
                        };
                        vis.mouse_input(&context.gl, button, state);
                    }
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    _ => (),
                },
                Event::RedrawRequested(_) => {
                    draw(&context.gl, &mut vis);
                    context.window.request_redraw();
                }
                _ => (),
            }
        });
    }
}

// contains gl context, window, event loop
#[cfg(not(target_arch = "wasm32"))]
pub struct VisContext {
    pub gl: glow::Context,
    pub shader_version: String,
    pub ctx: glutin::ContextWrapper<glutin::PossiblyCurrent, glutin::window::Window>,
    pub event_loop: glutin::event_loop::EventLoop<()>,
}

#[cfg(not(target_arch = "wasm32"))]
impl VisContext {
    pub fn new(width: f64, height: f64) -> Result<Self, VisContextError> {
        use native::*;
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_inner_size(LogicalSize::new(width, height))
            .with_title("window");
        let ctx_builder = ContextBuilder::new()
            .with_multisampling(4)
            .build_windowed(window, &event_loop)?;
        let ctx;
        let gl;
        unsafe {
            ctx = ctx_builder.make_current().unwrap();
            gl = glow::Context::from_loader_function(|x| ctx.get_proc_address(x) as *const _);
        }
        let shader_version = String::from("#version 410");
        Ok(Self {
            gl,
            shader_version,
            ctx,
            event_loop,
        })
    }

    // window passed as argument since running event loop causes move
    // calls vis event handlers on event
    pub fn run(window: VisContext, mut vis: VisGl) -> Result<(), VisContextError> {
        use native::*;
        vis.setup_gl_resources(&window.gl)?;
        let mut draw = VisGl::get_draw();
        window.event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CursorMoved { position, .. } => {
                        vis.mouse_move(&window.gl, position.x, position.y);
                    }
                    WindowEvent::MouseWheel { delta, .. } => {
                        let ds = match delta {
                            MouseScrollDelta::PixelDelta(position) => position.y,
                            MouseScrollDelta::LineDelta(_, y) => y as f64,
                        };
                        vis.mouse_wheel(&window.gl, ds);
                    }
                    WindowEvent::MouseInput { button, state, .. } => {
                        let button = match button {
                            MouseButtonGlutin::Left => MouseButton::Left,
                            MouseButtonGlutin::Right => MouseButton::Right,
                            _ => MouseButton::Other,
                        };
                        let state = match state {
                            ElementState::Pressed => true,
                            ElementState::Released => false,
                        };
                        vis.mouse_input(&window.gl, button, state);
                    }
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    _ => (),
                },
                Event::LoopDestroyed => {
                    vis.drop(&window.gl);
                }
                Event::RedrawRequested(_) => {
                    draw(&window.gl, &mut vis);
                    window.ctx.swap_buffers().unwrap();
                    window.ctx.window().request_redraw();
                }
                _ => (),
            }
        });
    }
}

use thiserror::Error;

#[derive(Error, Debug)]
pub enum VisContextError {
    #[error("{0}")]
    VisGl(#[from] VisGlError),
    #[error("{0}")]
    Mvp(#[from] MvpError),
    #[error("{0}")]
    Globe(#[from] crate::globe::GlobeError),
    #[error("{0}")]
    Attrib(#[from] crate::gl_wrap::AttribError),
    #[cfg(not(target_arch = "wasm32"))]
    #[error("{0}")]
    CtxCreation(#[from] glutin::CreationError),
    #[cfg(target_arch = "wasm32")]
    #[error("Web sys canvas retrieval failed")]
    Canvas,
    #[cfg(target_arch = "wasm32")]
    #[error("Web sys webgl2 context creation failed")]
    WebGl2Context,
}

#[derive(Error, Debug)]
pub enum MvpError {
    #[error("{0}")]
    UniformMatrix(#[from] crate::gl_wrap::UniformMatrixError),
}
