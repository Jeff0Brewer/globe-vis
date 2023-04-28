#[cfg(target_arch = "wasm32")]
use console_error_panic_hook::set_once as set_console_panic_hook;
mod gl_wrap;
mod globe;
mod icosphere;
mod mouse;
mod points;
mod vis;
mod vis_ctx;
use vis::VisBuilder;

pub type UpdateFn = fn(f32) -> Vec<f32>;

fn update(ms: f32) -> Vec<f32> {
    (0..300)
        .map(|x| (x as f32 * (ms % 1000.0) / 1000.0) / 300.0)
        .collect()
}

fn main() {
    // set panic hook for browser error logging
    #[cfg(target_arch = "wasm32")]
    set_console_panic_hook();

    VisBuilder::new()
        .with_dimensions(1000.0, 700.0)
        .with_update(update)
        .start()
        .unwrap();
}
