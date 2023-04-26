use crate::gl_wrap::Drop;
use crate::mouse::MouseButtons;
use crate::vis::{VisGl, VisGlError};

// use glutin when compiling to native
#[cfg(not(target_arch = "wasm32"))]
mod native {
    pub use glutin::{
        dpi::LogicalSize,
        event::{ElementState, Event, MouseButton, MouseScrollDelta, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::{Window, WindowBuilder},
        ContextBuilder, ContextWrapper, PossiblyCurrent,
    };
    pub type VisWindow = ContextWrapper<PossiblyCurrent, Window>;
}
#[cfg(not(target_arch = "wasm32"))]
use native::*;

// use winit when compiling to wasm
#[cfg(target_arch = "wasm32")]
mod web {
    pub use wasm_bindgen::JsCast;
    pub use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};
    pub use winit::{
        event::MouseButton as MouseButtonWinit,
        event::{ElementState, Event, MouseButton, MouseScrollDelta, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        platform::web::WindowExtWebSys,
        window::{Window, WindowBuilder},
    };
    pub type VisWindow = Window;
}
#[cfg(target_arch = "wasm32")]
use web::*;

// contains gl context and main event loop
pub struct VisContext {
    pub gl: glow::Context,
    pub event_loop: EventLoop<()>,
    pub shader_version: String,
    pub window: VisWindow,
}

impl VisContext {
    // native constructor, initialize glutin window and get context
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(width: f64, height: f64) -> Result<Self, VisContextError> {
        let shader_version = String::from("#version 410");
        let event_loop = EventLoop::new();
        let window_builder = WindowBuilder::new()
            .with_inner_size(LogicalSize::new(width, height))
            .with_title("window");
        let ctx_builder = ContextBuilder::new()
            .with_multisampling(4)
            .build_windowed(window_builder, &event_loop)?;
        let (gl, window);
        unsafe {
            window = ctx_builder.make_current().unwrap();
            gl = glow::Context::from_loader_function(|x| window.get_proc_address(x) as *const _);
        }
        Ok(Self {
            gl,
            window,
            event_loop,
            shader_version,
        })
    }

    // wasm constructor, init winit window, create canvas with webgl2 ctx and append to dom
    #[cfg(target_arch = "wasm32")]
    pub fn new(width: f64, height: f64) -> Result<Self, VisContextError> {
        let shader_version = String::from("#version 300 es");
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title("window")
            .build(&event_loop)?;
        let canvas = window.canvas();
        canvas
            .style()
            .set_css_text(&format!("width: {:.0}px; height: {:.0}px;", width, height));
        canvas.set_width(width as u32);
        canvas.set_height(height as u32);
        let ctx = canvas
            .get_context("webgl2")
            .ok()
            .and_then(|o| o)
            .and_then(|e| e.dyn_into::<WebGl2RenderingContext>().ok())
            .ok_or(VisContextError::WebGl2Context)?;
        let gl = glow::Context::from_webgl2_context(ctx);
        web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.body())
            .and_then(|b| b.append_child(&canvas).ok())
            .ok_or(VisContextError::DomBody)?;
        Ok(Self {
            gl,
            window,
            event_loop,
            shader_version,
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn redraw(window: &VisWindow) {
        window.swap_buffers().unwrap();
        window.window().request_redraw();
    }

    #[cfg(target_arch = "wasm32")]
    fn redraw(window: &VisWindow) {
        window.request_redraw();
    }

    // window passed as argument since running event loop causes move
    // calls vis event handlers on event
    pub fn run(context: VisContext, mut vis: VisGl) -> Result<(), VisContextError> {
        vis.setup_gl_resources(&context.gl)?;
        let mut draw = VisGl::get_draw();

        context.event_loop.run(move |event, _, control_flow| {
            #[cfg(not(target_arch = "wasm32"))]
            control_flow.set_wait();
            #[cfg(target_arch = "wasm32")]
            control_flow.set_poll();

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
                            MouseButton::Left => MouseButtons::Left,
                            MouseButton::Right => MouseButtons::Right,
                            _ => MouseButtons::Other,
                        };
                        let state = match state {
                            ElementState::Pressed => true,
                            ElementState::Released => false,
                        };
                        vis.mouse_input(&context.gl, button, state);
                    }
                    WindowEvent::CloseRequested => control_flow.set_exit(),
                    _ => (),
                },
                Event::LoopDestroyed => {
                    vis.drop(&context.gl);
                }
                Event::RedrawRequested(_) => {
                    draw(&context.gl, &mut vis);
                    VisContext::redraw(&context.window);
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
    #[error("Canvas element couldn't be added to web sys body")]
    DomBody,
    #[cfg(target_arch = "wasm32")]
    #[error("Web sys webgl2 context creation failed")]
    WebGl2Context,
    #[cfg(target_arch = "wasm32")]
    #[error("{0}")]
    Os(#[from] winit::error::OsError),
}
