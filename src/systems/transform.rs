use glam::Vec4;

pub struct Transform {
    pub position: Vec4,
}

impl Transform {

    pub fn new_zero() -> Self {
        Transform { position: Vec4::ZERO }
    }

    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Transform { position: Vec4::new(x, y, z, w) }
    }

    pub fn add_position(&mut self, addition: Vec4) {
        self.position += addition;
    }

    pub fn get_position(&self) -> Vec4 {
        self.position
    }

    pub fn get_direction(&mut self) -> Vec4 {
        Vec4::ZERO
    }
}

pub struct Position {

}