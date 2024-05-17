use std::collections::HashSet;
use crate::physics::hitbox::HitBox;
use crate::render::types::{len_vec3f, norm_vec3f, sub_vec3f, Vec3f, Vec3i};
use crate::world::World;

const E: f32 = 0.015625; //ERROR

pub struct PhysicsSolver {
    gravity: Vec3f,
}

impl PhysicsSolver {
    pub fn new(gravity: Vec3f) -> Self {
        return Self {gravity};
    }

    pub fn step(&self, world: &World, hitbox: &mut HitBox, delta: f32, steps_cnt: u32) {
        let mut ltmppos: Vec3f = [hitbox.pos[0], hitbox.pos[1], hitbox.pos[2]];
        let step: f32 = delta / (steps_cnt as f32);
        for ind in 0_u32..steps_cnt {
            let mut pos: &mut Vec3f = &mut hitbox.pos;
            let mut vel: &mut Vec3f = &mut hitbox.vel;
            let hsz: &Vec3f = &hitbox.half_size;
            vel[0] += self.gravity[0] * step;
            vel[1] += self.gravity[1] * step;
            vel[2] += self.gravity[2] * step;

            let last_pos_x: f32 = pos[0];
            let last_pos_z: f32 = pos[2];



            //x
            if vel[0] < 0.0 {
                for y_pos in ((pos[1] - hsz[1] + E).floor() as i32)..=((pos[1] + hsz[1] - E).floor() as i32) {
                    for z_pos in ((pos[2] - hsz[2] + E).floor() as i32)..=((pos[2] + hsz[2] - E).floor() as i32) {
                        let x_pos: i32 = ((pos[0] - hsz[0] - E).floor() as i32);
                        if world.is_obstacle(&[x_pos, y_pos, z_pos]) {
                            vel[0] = 0.0;
                            pos[0] = (x_pos as f32) + 1.0 + hsz[0] + E;
                            break;
                        }
                    }
                }
            }
            if vel[0] > 0.0 {
                for y_pos in ((pos[1] - hsz[1] + E).floor() as i32)..=((pos[1] + hsz[1] - E).floor() as i32) {
                    for z_pos in ((pos[2] - hsz[2] + E).floor() as i32)..=((pos[2] + hsz[2] - E).floor() as i32) {
                        let x_pos = ((pos[0] + hsz[0] + E).floor() as i32);
                        if world.is_obstacle(&[x_pos, y_pos, z_pos]) {
                            vel[0] = 0.0;
                            pos[0] = ((x_pos as f32) - hsz[0]) - E;
                            break;
                        }
                    }
                }
            }
            //

            //z
            if vel[2] < 0.0 {
                for y_pos in ((pos[1] - hsz[1] + E).floor() as i32)..=((pos[1] + hsz[1] - E).floor() as i32) {
                    for x_pos in ((pos[0] - hsz[0] + E).floor() as i32)..=((pos[0] + hsz[0] - E).floor() as i32) {
                        let z_pos: i32 = ((pos[2] - hsz[2] - E).floor() as i32);
                        if world.is_obstacle(&[x_pos, y_pos, z_pos]) {
                            vel[2] = 0.0;
                            pos[2] = (z_pos as f32) + 1.0 + hsz[2] + E;
                            break;
                        }
                    }
                }
            }
            if vel[2] > 0.0 {
                for y_pos in ((pos[1] - hsz[1] + E).floor() as i32)..=((pos[1] + hsz[1] - E).floor() as i32) {
                    for x_pos in ((pos[0] - hsz[0] + E).floor() as i32)..=((pos[0] + hsz[0] - E).floor() as i32) {
                        let z_pos = ((pos[2] + hsz[2] + E).floor() as i32);
                        if world.is_obstacle(&[x_pos, y_pos, z_pos]) {
                            vel[2] = 0.0;
                            pos[2] = ((z_pos as f32) - hsz[2]) - E;
                            break;
                        }
                    }
                }
            }
            //

            //y
            hitbox.grounded = false;
            if vel[1] < 0.0 {
                'end:
                for x_pos in ((pos[0] - hsz[0] + E).floor() as i32)..=((pos[0] + hsz[0] - E).floor() as i32) {
                    for z_pos in ((pos[2] - hsz[2] + E).floor() as i32)..=((pos[2] + hsz[2] - E).floor() as i32) {
                        let y_pos: i32 = ((pos[1] - hsz[1] - E).floor() as i32);
                        if world.is_obstacle(&[x_pos, y_pos, z_pos]) {
                            vel[1] = 0.0;
                            pos[1] = (y_pos as f32) + 1.0 + hsz[1];
                            let f: f32 = 8.0; //friction
                            vel[0] *= 0.0_f32.max(1.0 - (step * f));
                            vel[2] *= 0.0_f32.max(1.0 - (step * f));
                            hitbox.grounded = true;
                            break 'end;
                        }
                    }
                }
            }
            if vel[1] > 0.0 {
                for x_pos in ((pos[0] - hsz[0] + E).floor() as i32)..=((pos[0] + hsz[0] - E).floor() as i32) {
                    for z_pos in ((pos[2] - hsz[2] + E).floor() as i32)..=((pos[2] + hsz[2] - E).floor() as i32) {
                        let y_pos = ((pos[1] + hsz[1] + E).floor() as i32);
                        if world.is_obstacle(&[x_pos, y_pos, z_pos]) {
                            vel[1] = 0.0;
                            pos[1] = ((y_pos as f32) - hsz[1]) + E;
                            break;
                        }
                    }
                }
            }
            //

            pos[0] += vel[0] * step;
            pos[1] += vel[1] * step;
            pos[2] += vel[2] * step;

            if hitbox.shifting && hitbox.grounded {
                let y_pos: i32 = ((pos[1] - hsz[1]) - E).floor() as i32;
                hitbox.grounded = false;

                for x_pos in ((last_pos_x - hsz[0] + E).floor() as i32)..=((last_pos_x + hsz[0] - E).floor() as i32) {
                    for z_pos in ((pos[2] - hsz[2] + E).floor() as i32)..=((pos[2] + hsz[2] - E).floor() as i32) {
                        if world.is_obstacle(&[x_pos, y_pos, z_pos]) {
                            hitbox.grounded = true;
                            break;
                        }
                    }
                }
                if !hitbox.grounded {pos[2] = last_pos_z;}

                for x_pos in ((pos[0] - hsz[0] + E).floor() as i32)..=((pos[0] + hsz[0] - E).floor() as i32) {
                    for z_pos in ((last_pos_z - hsz[2] + E).floor() as i32)..=((last_pos_z + hsz[2] - E).floor() as i32) {
                        if world.is_obstacle(&[x_pos, y_pos, z_pos]) {
                            hitbox.grounded = true;
                            break;
                        }
                    }
                }
                if !hitbox.grounded {
                    pos[0] = last_pos_x;
                }
                hitbox.grounded = true;
            }
        }
    }

    //TODO CALLING VIA GETTERS AND SETTERS
    pub fn is_block_inside(pos: &Vec3i, hitbox: &HitBox) -> bool {
        let hpos: &Vec3f = &hitbox.pos;
        let hsz: &Vec3f = &hitbox.half_size;
        return pos[0] >= ((hpos[0] - hsz[0]).floor() as i32) && pos[0] <= ((hpos[0] + hsz[0]).floor() as i32) &&
                pos[1] >= ((hpos[1] - hsz[1]).floor() as i32) && pos[1] <= ((hpos[1] + hsz[1]).floor() as i32) &&
                pos[2] >= ((hpos[2] - hsz[2]).floor() as i32) && pos[2] <= ((hpos[2] + hsz[2]).floor() as i32);
    }
}