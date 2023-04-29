use crate::{
    vis_ctx::{VisContext, VisContextError},
    vis_gl::{VisGl, VisGlError},
    VisState,
};

// builder for initialization and running vis
pub struct VisBuilder<T: VisState + 'static> {
    width: Option<f64>,
    height: Option<f64>,
    state: Option<T>,
}

impl<T: VisState + 'static> VisBuilder<T> {
    pub fn new() -> Self {
        let width = None;
        let height = None;
        let state = None;
        Self {
            width,
            height,
            state,
        }
    }

    // set window size
    pub fn with_dimensions(mut self, width: f64, height: f64) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    // add user state
    pub fn with_state(mut self, state: T) -> Self {
        self.state = Some(state);
        self
    }

    // run visualization from prev set fields
    pub fn start(&mut self) -> Result<(), VisError> {
        let width = self.width.unwrap_or(500.0);
        let height = self.height.unwrap_or(500.0);
        let state = self.state.take();

        let window = VisContext::new(width, height)?;
        let gl = VisGl::new(&window, width, height)?;
        VisContext::run(window, gl, state)?;
        Ok(())
    }
}

use thiserror::Error;

#[derive(Error, Debug)]
pub enum VisError {
    #[error("{0}")]
    VisGl(#[from] VisGlError),
    #[error("{0}")]
    VisContext(#[from] VisContextError),
}
