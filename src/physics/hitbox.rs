use crate::render::types::Vec3f;

pub struct HitBox {
    pub pos: Vec3f,
    pub vel: Vec3f,
    pub half_size: Vec3f,
    pub grounded: bool,
    pub shifting: bool,
}

impl HitBox {
    pub fn new(position: Vec3f, half_size: Vec3f) -> Self {
        return Self {
            pos: position,
            vel: [0.0, 0.0, 0.0],
            half_size,
            grounded: false,
            shifting: false,
        };
    }
}