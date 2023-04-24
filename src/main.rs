mod gl_wrap;
mod globe;
mod icosphere;
mod mouse;
mod vis;
use vis::Vis;

fn main() {
    #[cfg(target_arch = "wasm32")]
    use console_error_panic_hook::set_once as set_panic_hook;
    #[cfg(target_arch = "wasm32")]
    set_panic_hook();

    let vis = Vis::new(500.0, 500.0).unwrap();
    Vis::start(vis).unwrap();
}
