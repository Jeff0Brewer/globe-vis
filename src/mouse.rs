use glam::{Mat4, Quat, Vec3};

pub struct MouseState {
    pub x: f64,
    pub y: f64,
    pub dragging: bool,
}

impl MouseState {
    pub fn new() -> Self {
        MouseState {
            x: 0.0,
            y: 0.0,
            dragging: false,
        }
    }
}

pub enum MouseButton {
    Left,
    Right,
    Other,
}

const ROT_SPEED: f64 = 0.05;
const ZOOM_SPEED: f64 = 0.03;

pub fn rotate_from_mouse(mat: Mat4, dx: f64, dy: f64) -> Mat4 {
    let x_rad = (dy * ROT_SPEED) as f32;
    let y_rad = (dx * ROT_SPEED) as f32;

    // transform x / y axis by inverse of current rotation
    // to get rotation axis perpendicular to current view
    let inv_mat = mat.inverse();
    let x_axis = inv_mat.transform_vector3(Vec3::X);
    let y_axis = inv_mat.transform_vector3(Vec3::Y);

    let x_rot = Mat4::from_quat(Quat::from_axis_angle(x_axis, x_rad));
    let y_rot = Mat4::from_quat(Quat::from_axis_angle(y_axis, y_rad));

    mat.mul_mat4(&x_rot.mul_mat4(&y_rot))
}

pub fn zoom_from_scroll(mat: Mat4, delta: f64) -> Mat4 {
    let zoom = (delta * ZOOM_SPEED) as f32;
    let scale = Mat4::from_scale(Vec3::splat(1.0 + zoom));
    mat.mul_mat4(&scale)
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Mat4, Vec3};

    fn assert_matrix_elements_near_eq(a: Mat4, b: Mat4, epsilon: f32) {
        let a = a.to_cols_array();
        let b = b.to_cols_array();
        for i in 0..16 {
            assert!(
                (a[i] - b[i]).abs() < epsilon,
                "Matrices not equal at element {}: {} != {}",
                i,
                a[i],
                b[i]
            );
        }
    }

    #[test]
    fn test_rotate_from_mouse() {
        let mat = Mat4::IDENTITY;
        let dx = 10.0;
        let dy = 20.0;

        let rotated_mat = rotate_from_mouse(mat, dx, dy);
        let x_rad = (dy * ROT_SPEED) as f32;
        let y_rad = (dx * ROT_SPEED) as f32;
        let expected_x_rotation = Mat4::from_rotation_x(x_rad);
        let expected_y_rotation = Mat4::from_rotation_y(y_rad);

        let expected_mat = mat.mul_mat4(&expected_x_rotation.mul_mat4(&expected_y_rotation));

        assert_matrix_elements_near_eq(rotated_mat, expected_mat, 1e-6);
    }

    #[test]
    fn test_zoom_from_scroll() {
        let mat = Mat4::IDENTITY;
        let delta = 30.0;

        let zoomed_mat = zoom_from_scroll(mat, delta);
        let expected_zoom = 1.0 + (delta * ZOOM_SPEED) as f32;
        let expected_scale = Mat4::from_scale(Vec3::splat(expected_zoom));
        let expected_mat = mat.mul_mat4(&expected_scale);

        assert_matrix_elements_near_eq(zoomed_mat, expected_mat, 1e-6);
    }
}
