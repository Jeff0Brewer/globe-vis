#[cfg(target_arch = "wasm32")]
use console_error_panic_hook::set_once as set_console_panic_hook;
mod gl_wrap;
mod globe;
mod icosphere;
mod mouse;
mod vis;
mod vis_ctx;
use vis::Vis;

fn main() {
    // set panic hook for browser error logging
    #[cfg(target_arch = "wasm32")]
    set_console_panic_hook();

    let vis = Vis::new(1000.0, 700.0).unwrap();
    Vis::start(vis).unwrap();
}
