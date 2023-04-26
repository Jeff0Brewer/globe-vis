use crate::gl_wrap::Drop;
use crate::mouse::MouseButton;
use crate::vis::{VisGl, VisGlError};
#[cfg(not(target_arch = "wasm32"))]
mod native {
    pub use glutin::{
        dpi::LogicalSize,
        event::MouseButton as MouseButtonGlutin,
        event::{ElementState, Event, MouseScrollDelta, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::{Window, WindowBuilder},
        ContextBuilder, ContextWrapper, PossiblyCurrent,
    };
}
#[cfg(not(target_arch = "wasm32"))]
use native::*;

#[cfg(target_arch = "wasm32")]
mod web {
    pub use wasm_bindgen::JsCast;
    pub use web_sys::{window, HtmlCanvasElement, WebGl2RenderingContext};
    pub use winit::{
        event::MouseButton as MouseButtonWinit,
        event::{ElementState, Event, MouseScrollDelta, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        platform::web::WindowExtWebSys,
        window::{Window, WindowBuilder},
    };
}
#[cfg(target_arch = "wasm32")]
use web::*;

// contains gl context, window, event loop
#[cfg(not(target_arch = "wasm32"))]
pub struct VisContext {
    pub gl: glow::Context,
    pub window: ContextWrapper<PossiblyCurrent, Window>,
    pub event_loop: EventLoop<()>,
    pub shader_version: String,
}

#[cfg(target_arch = "wasm32")]
pub struct VisContext {
    pub gl: glow::Context,
    pub window: Window,
    pub event_loop: EventLoop<()>,
    pub shader_version: String,
}

#[cfg(not(target_arch = "wasm32"))]
impl VisContext {
    pub fn new(width: f64, height: f64) -> Result<Self, VisContextError> {
        let event_loop = EventLoop::new();
        let window_builder = WindowBuilder::new()
            .with_inner_size(LogicalSize::new(width, height))
            .with_title("window");
        let ctx_builder = ContextBuilder::new()
            .with_multisampling(4)
            .build_windowed(window_builder, &event_loop)?;
        let gl;
        let window;
        unsafe {
            window = ctx_builder.make_current().unwrap();
            gl = glow::Context::from_loader_function(|x| window.get_proc_address(x) as *const _);
        }
        let shader_version = String::from("#version 410");
        Ok(Self {
            gl,
            window,
            event_loop,
            shader_version,
        })
    }

    // window passed as argument since running event loop causes move
    // calls vis event handlers on event
    pub fn run(context: VisContext, mut vis: VisGl) -> Result<(), VisContextError> {
        vis.setup_gl_resources(&context.gl)?;
        let mut draw = VisGl::get_draw();
        context.event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;
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
                            MouseButtonGlutin::Left => MouseButton::Left,
                            MouseButtonGlutin::Right => MouseButton::Right,
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
                Event::LoopDestroyed => {
                    vis.drop(&context.gl);
                }
                Event::RedrawRequested(_) => {
                    draw(&context.gl, &mut vis);
                    context.window.swap_buffers().unwrap();
                    context.window.window().request_redraw();
                }
                _ => (),
            }
        });
    }
}

#[cfg(target_arch = "wasm32")]
impl VisContext {
    pub fn new(width: f64, height: f64) -> Result<Self, VisContextError> {
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

use thiserror::Error;

#[derive(Error, Debug)]
pub enum VisContextError {
    #[error("{0}")]
    VisGl(#[from] VisGlError),
    #[cfg(not(target_arch = "wasm32"))]
    #[error("{0}")]
    CtxCreation(#[from] glutin::CreationError),
    #[cfg(target_arch = "wasm32")]
    #[error("Web sys webgl2 context creation failed")]
    WebGl2Context,
}

#[derive(Error, Debug)]
pub enum MvpError {
    #[error("{0}")]
    UniformMatrix(#[from] crate::gl_wrap::UniformMatrixError),
}
