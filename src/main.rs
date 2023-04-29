#[cfg(target_arch = "wasm32")]
use console_error_panic_hook::set_once as set_console_panic_hook;
mod gl_wrap;
mod globe;
mod icosphere;
mod mouse;
mod points;
mod state;
mod vis_build;
mod vis_ctx;
mod vis_gl;
use state::VisState;
use vis_build::VisBuilder;

pub struct TestState {
    modulus: f32,
    last_ms: f32,
}

impl TestState {
    pub fn new(modulus: f32) -> Self {
        let last_ms = 0.0;
        Self { modulus, last_ms }
    }
}

impl VisState for TestState {
    fn update_points(&mut self, ms: f32) -> Vec<f32> {
        let elapsed = ms - self.last_ms;
        self.modulus += 0.1 * elapsed;
        self.last_ms = ms;
        (0..300)
            .map(|x| (x as f32 * (ms % self.modulus) / self.modulus) / 300.0)
            .collect()
    }
}

fn main() {
    // set panic hook for browser error logging
    #[cfg(target_arch = "wasm32")]
    set_console_panic_hook();

    let state = TestState::new(1000.0);

    VisBuilder::new()
        .with_dimensions(1000.0, 700.0)
        .with_state(state)
        .start()
        .unwrap();
}
