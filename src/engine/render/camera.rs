use glam::{Mat4, Vec4};

pub struct Camera {
    pub rotation_matrix: Mat4,
    pub zw_rotation_matrix: Mat4,
    pub zy_rotation_matrix: Mat4,
    pub zx_rotation_matrix: Mat4,
    pub position: Vec4,
}

impl Camera {
    pub fn get_rotation_matrix(&self) -> Mat4
    {
        self.rotation_matrix
    }

    pub fn get_zw_rotation_matrix(&self) -> Mat4
    {
        self.zw_rotation_matrix
    }

    pub fn get_zy_rotation_matrix(&self) -> Mat4
    {
        self.zy_rotation_matrix
    }

    pub fn get_zx_rotation_matrix(&self) -> Mat4
    {
        self.zx_rotation_matrix
    }

    pub fn get_position(&self) -> Vec4
    {
        self.position
    }
}