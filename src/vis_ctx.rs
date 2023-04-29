use crate::{
    gl_wrap::Drop,
    mouse::{MouseButtons, SCROLL_LINE_HEIGHT},
    vis_gl::{VisGl, VisGlError},
    VisState,
};
use glow::HasContext;
use instant::Instant;

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
    pub dpi: f64,
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
            window = ctx_builder
                .make_current()
                .map_err(|_| VisContextError::CtxCurrent)?;
            gl = glow::Context::from_loader_function(|x| window.get_proc_address(x) as *const _);
        }
        let dpi = window.window().scale_factor();
        Ok(Self {
            gl,
            window,
            event_loop,
            shader_version,
            dpi,
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
        let dpi = window.scale_factor();
        let canvas = window.canvas();
        canvas
            .style()
            .set_css_text(&format!("width: {:.0}px; height: {:.0}px;", width, height));
        canvas.set_width((width * dpi) as u32);
        canvas.set_height((height * dpi) as u32);
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
            dpi,
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
    pub fn run<T: VisState + 'static>(
        mut context: VisContext,
        mut vis: VisGl,
        mut state: Option<T>,
    ) -> Result<(), VisContextError> {
        vis.setup_gl(&context.gl)?;

        let time = Instant::now();
        context.event_loop.run(move |event, _, control_flow| {
            #[cfg(not(target_arch = "wasm32"))]
            control_flow.set_wait();
            #[cfg(target_arch = "wasm32")]
            control_flow.set_poll();

            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CursorMoved { position, .. } => {
                        vis.mouse_move(&context.gl, position.x, position.y).unwrap();
                    }
                    WindowEvent::MouseWheel { delta, .. } => {
                        let ds = match delta {
                            MouseScrollDelta::PixelDelta(position) => position.y / context.dpi,
                            MouseScrollDelta::LineDelta(_, y) => (y as f64) * SCROLL_LINE_HEIGHT,
                        };
                        vis.mouse_wheel(&context.gl, ds).unwrap();
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
                    WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                        context.dpi = scale_factor
                    }
                    WindowEvent::CloseRequested => {
                        control_flow.set_exit();
                    }
                    _ => (),
                },
                Event::LoopDestroyed => {
                    vis.drop(&context.gl);
                }
                Event::RedrawRequested(_) => {
                    let elapsed = time.elapsed().as_millis() as f32;
                    let point_data = state.as_mut().map(|u| u.update_points(elapsed));

                    unsafe {
                        context
                            .gl
                            .clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
                    }
                    vis.globe.draw(&context.gl);
                    vis.points.draw(&context.gl, point_data);
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
    #[cfg(not(target_arch = "wasm32"))]
    #[error("Context could not be made current")]
    CtxCurrent,
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
