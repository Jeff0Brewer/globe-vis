extern crate gl;
extern crate glutin;
mod gl_wrap;
mod globe;
mod icosphere;
mod mouse;
mod vis;
use vis::Vis;

fn main() {
    let vis = Vis::new(500.0, 500.0);
    Vis::start(vis);
}
