use glam::{Mat4, Quat, Vec3};

pub fn rotate_from_mouse(mat: Mat4, dx: f64, dy: f64) -> Mat4 {
    let rotation_speed = 0.05;
    let x_rad = (dy * rotation_speed) as f32;
    let y_rad = (dx * rotation_speed) as f32;

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
    let zoom = (delta * 0.03) as f32;
    let scale = Mat4::from_scale(Vec3::splat(1.0 + zoom));
    mat.mul_mat4(&scale)
}
