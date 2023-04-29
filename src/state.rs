pub trait VisState {
    fn update_points(&mut self, ms: f32) -> Vec<f32>;
}
