use crate::gl_wrap::{Buffer, Drop, Program};
use crate::icosphere::get_icosphere;

pub struct Globe {
    pub data: Vec<f32>,
    pub program: Program,
    pub buffer: Buffer,
}

impl Globe {
    pub fn new() -> Self {
        let data = get_icosphere(4);
        let buffer = Buffer::new(&data, gl::DYNAMIC_DRAW);
        let program =
            Program::new_from_files("./shaders/vert.glsl", "./shaders/frag.glsl").unwrap();
        Self {
            program,
            buffer,
            data,
        }
    }

    pub fn get_draw() -> impl FnMut(&mut Globe) {
        let mut buf_change = 1.0;
        let mut buf_change_dir = 1.0;
        move |globe: &mut Globe| {
            if !(0.5..1.0).contains(&buf_change) {
                buf_change_dir = -buf_change_dir;
            }
            buf_change += buf_change_dir * 0.001;
            let data: Vec<f32> = globe.data.iter().map(|x| x * buf_change).collect();
            globe.buffer.set_data(&data);
            unsafe {
                gl::DrawArrays(gl::TRIANGLES, 0, (data.len() / 3) as i32);
            }
        }
    }
}

impl Drop for Globe {
    fn drop(&self) {
        self.program.drop();
        self.buffer.drop();
    }
}
