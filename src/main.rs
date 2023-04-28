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

fn main() {
    // set panic hook for browser error logging
    #[cfg(target_arch = "wasm32")]
    set_console_panic_hook();

    VisBuilder::new()
        .with_dimensions(1000.0, 700.0)
        .start()
        .unwrap();
}
