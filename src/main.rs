mod gl_wrap;
mod globe;
mod icosphere;
mod mouse;
mod points;
mod state;
mod vis_build;
mod vis_ctx;
mod vis_gl;

#[cfg(target_arch = "wasm32")]
use console_error_panic_hook::set_once as set_console_panic_hook;
use state::VisState;
use vis_build::VisBuilder;

pub struct TestState {
    offsets: Vec<f32>,
    positions: Vec<f32>,
    length: usize,
}

impl TestState {
    pub fn new(length: usize) -> Self {
        let offsets = (0..length)
            .map(|x| 2.0 * std::f32::consts::PI * (x as f32 / length as f32))
            .collect();
        let positions = (0..length)
            .map(|x| (x as f32 / (length as f32 / 10.0)) % 1.0 - 0.5)
            .collect();
        Self {
            offsets,
            positions,
            length,
        }
    }
}

const SPEED: f32 = std::f32::consts::PI / 10000.0;
const RADIUS: f32 = 1.5;
impl VisState for TestState {
    fn update_points(&mut self, ms: f32) -> Vec<f32> {
        (0..self.length)
            .map(|i| match i % 3 {
                0 => RADIUS * (self.offsets[i] + ms * SPEED).cos(),
                1 => self.positions[i],
                2 => RADIUS * (self.offsets[i] + ms * SPEED).sin(),
                _ => -1.0,
            })
            .collect()
    }
}

fn main() {
    // set panic hook for browser error logging
    #[cfg(target_arch = "wasm32")]
    set_console_panic_hook();

    let state = TestState::new(100000);

    VisBuilder::new()
        .with_dimensions(1000.0, 700.0)
        .with_state(state)
        .start()
        .unwrap();
}
