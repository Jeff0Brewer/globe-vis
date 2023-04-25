#[cfg(target_arch = "wasm32")]
use console_error_panic_hook::set_once as set_console_panic_hook;
mod gl_wrap;
mod globe;
mod icosphere;
mod mouse;
mod vis;
use vis::Vis;

fn main() {
    // set panic hook for browser error logging
    #[cfg(target_arch = "wasm32")]
    set_console_panic_hook();

    let vis = Vis::new(500.0, 500.0).unwrap();
    Vis::start(vis).unwrap();
}
