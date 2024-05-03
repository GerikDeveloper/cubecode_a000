use std::f32::consts;
use std::f32::consts::{PI, TAU};
use crate::render::types::{Mat4f, Vec3f, Vec4f};

//TODO REWRITE ALL

pub struct Camera {
    pos: Vec3f,
    rot: Vec3f,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            pos: [0.0, 0.0, 0.0],
            rot: [0.0, 0.0, 0.0],
        }
    }

    pub fn from(pos: Vec3f, rot: Vec3f) -> Self {
        Self {
            pos,
            rot,
        }
    }

    pub fn set_position_xyz(&mut self, x: f32, y: f32, z: f32) {
        self.pos[0] = x;
        self.pos[1] = y;
        self.pos[2] = z;
    }

    pub fn set_position_x(&mut self, x: f32) {
        self.pos[0] = x;
    }

    pub fn set_position_y(&mut self, y: f32) {
        self.pos[1] = y;
    }

    pub fn set_position_z(&mut self, z: f32) {
        self.pos[2] = z;
    }

    pub fn set_position(&mut self, pos: Vec3f) {
        self.pos = pos;
    }

    pub fn get_position(&self) -> Vec3f {
        return self.pos;
    }

    pub fn move_position(&mut self, offset: &Vec3f) {
        if offset[2] != 0.0 {
            self.pos[0] += -(self.rot[1].sin() * offset[2]);
            self.pos[2] += self.rot[1].cos() * offset[2];
        }
        if offset[0] != 0.0 {
            self.pos[0] += -((self.rot[1] - (consts::FRAC_PI_2)).sin() * offset[0]);
            self.pos[2] += (self.rot[1] - (consts::FRAC_PI_2)).cos() * offset[0];
        }
        self.pos[1] += offset[1];
    }

    pub fn set_rotation_xyz(&mut self, x: f32, y: f32, z: f32) {
        self.rot[0] = x.to_radians();
        self.rot[1] = y.to_radians();
        self.rot[2] = z.to_radians();
    }

    pub fn set_rotation_x(&mut self, x: f32) {
        self.rot[0] = x.to_radians();
    }

    pub fn set_rotation_y(&mut self, y: f32) {
        self.rot[1] = y.to_radians();
    }

    pub fn set_rotation_z(&mut self, z: f32) {
        self.rot[2] = z.to_radians();
    }

    pub fn set_rotation(&mut self, rot: Vec3f) {
        self.rot = [rot[0].to_radians(), rot[1].to_radians(), rot[2].to_radians()];
    }

    pub fn get_rotation(&self) -> Vec3f {
        return [self.rot[0].to_degrees(), self.rot[1].to_degrees(), self.rot[2].to_radians()];
    }

    pub fn get_rotation_x(&self) -> f32 {
        return self.rot[0].to_degrees();
    }

    pub fn get_rotation_y(&self) -> f32 {
        return self.rot[1].to_degrees();
    }

    pub fn get_rotation_z(&self) -> f32 {
        return self.rot[2].to_degrees();
    }

    pub fn move_rotation(&mut self, rot: &Vec3f) {
        self.rot[0] += rot[0].to_radians();
        self.rot[1] += rot[1].to_radians();
        self.rot[2] += rot[2].to_radians();
        if self.rot[0] > TAU {
            self.rot[0] %= TAU;
        }
        if self.rot[1] > TAU {
            self.rot[1] %= TAU;
        }
        if self.rot[2] > TAU {
            self.rot[2] %= TAU;
        }
    }

    pub fn get_view_mat(&self, proj_mat: &Mat4f) -> Mat4f {
        let mut res = Mat4f::new();
        res.identity()
            .rotate(&self.rot)
            .translate_xyz(-self.pos[0], -self.pos[1], -self.pos[2])
            .mul(&proj_mat);
        res
    }

    //TODO TRANSLATE s, x, y, z
    pub fn get_view_mat_to(&self, proj_mat: &Mat4f, view_mat: &mut Mat4f) {
        view_mat.identity()
            .rotate(&self.rot)
            .translate_xyz(-self.pos[0], -self.pos[1], -self.pos[2])
            .mul(&proj_mat);
    }

    pub fn get_dir(&self) -> Vec3f {
        let res: Vec4f = Mat4f::new().identity().rotate(&self.rot).mul_vec4f([0.0f32, 0.0f32, -1.0f32, 1.0f32]);
        return [res[0], res[1], res[2]];
    }
}