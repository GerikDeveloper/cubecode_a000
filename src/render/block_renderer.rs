use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use crate::render::blocks_loader::{Block, BlocksLoader, BlockUsingError, GRASS_BLOCK_ID, UNKNOWN_BLOCK_ID};
use crate::render::faces_loader::Face;
use crate::render::light::light_map::{B_CHANNEL, G_CHANNEL, R_CHANNEL, S_CHANNEL};
use crate::render::light::light_solver::LightSolver;
use crate::render::meshes_loader::Mesh::{Cube, Custom};
use crate::render::types::{Vec3b, Vec3s, Vec3ub, LightedTexVertex, RGBSColor};
use crate::world::World;

//TODO glEnable(CULL_FACE)

const NEIGHBOR_TOP: Vec3b =     [ 0, 1, 0];
const NEIGHBOR_BOTTOM: Vec3b =  [ 0,-1, 0];
const NEIGHBOR_FRONT: Vec3b =   [ 0, 0,-1];
const NEIGHBOR_BACK: Vec3b =    [ 0, 0, 1];
const NEIGHBOR_RIGHT: Vec3b =   [ 1, 0, 0];
const NEIGHBOR_LEFT: Vec3b =    [-1, 0, 0];

const NEIGHBORHOOD: [Vec3b; 6] = [NEIGHBOR_TOP, NEIGHBOR_BOTTOM, NEIGHBOR_FRONT, NEIGHBOR_BACK, NEIGHBOR_RIGHT, NEIGHBOR_LEFT];

fn get_neighbor_block(world: &World, pos: &Vec3ub, offset: &Vec3b) -> Option<u16> {
    let exp_sum: Vec3s = [(offset[0] as i16) + (pos[0] as i16), (offset[1] as i16) + (pos[1] as i16), (offset[2] as i16) + (pos[2] as i16)];
    if exp_sum[0] >= 0x00 && exp_sum[0] <= 0xFF &&
        exp_sum[1] >= 0x00 && exp_sum[1] <= 0xFF &&
        exp_sum[2] >= 0x00 && exp_sum[2] <= 0xFF {
        return Some(world.get_block(&[exp_sum[0] as u8, exp_sum[1] as u8, exp_sum[2] as u8]));
    }
    return None;
}

fn render_face(face: &Rc<Face>, vertices: &mut Vec<LightedTexVertex>, indices: &mut Vec<i32>, pos: &Vec3ub, light: &RGBSColor) {
    if face.indices.len() != 0 {
        let ind_offset = (vertices.len() as i32);
        for vertex in &face.vertices {
            let rend_vert: LightedTexVertex = LightedTexVertex([vertex.0[0] + (pos[0] as f32), vertex.0[1] + (pos[1] as f32), vertex.0[2] + (pos[2] as f32)], vertex.1, light.clone());
            vertices.push(rend_vert);
        }
        for ind in &face.indices {
            indices.push(ind + ind_offset);
        }
    }
}

pub(crate) fn get_neighbor_light(world: &World, pos: &Vec3ub, offset: &Vec3b, channel: u8) -> u8 { //if not exists return 0
    return if let Some(neigh_pos) = LightSolver::get_neighbor_pos(&pos, &offset) {
        world.get_light_level(&neigh_pos, channel)
    } else {
        0
    }
}

pub(crate) fn render_block(world: &World, blocks_loader: &BlocksLoader, block_lid: u16, vertices: &mut Vec<LightedTexVertex>, indices: &mut Vec<i32>, subchunk_pos: &Vec3ub, pos: &Vec3ub) -> Result<(), Box<dyn std::error::Error>> {
    let block: &Rc<Block> = match blocks_loader.loaded_blocks.get(block_lid as usize) {
        None => {
            if let Some(block) = blocks_loader.blocks_ids.get(&UNKNOWN_BLOCK_ID) {
                block
            } else {
                return Err(Box::new(BlockUsingError::BlockNotFoundError()));
            }
        }
        Some(block) => {block}
    };
    let global_pos: &Vec3ub = &[(pos[0] + (subchunk_pos[1] << 4)), (pos[1] + (subchunk_pos[0] << 4)), (pos[2] + (subchunk_pos[2] << 4))];
    match block.mesh.deref() {
        Cube(cube_mesh) => {
            let neighboring_faces: [(Vec3b, Rc<Face>); 6] = [
                (NEIGHBOR_TOP,      cube_mesh.top.clone()),
                (NEIGHBOR_BOTTOM,   cube_mesh.bottom.clone()),
                (NEIGHBOR_FRONT,    cube_mesh.front.clone()),
                (NEIGHBOR_BACK,     cube_mesh.back.clone()),
                (NEIGHBOR_RIGHT,    cube_mesh.right.clone()),
                (NEIGHBOR_LEFT,     cube_mesh.left.clone()),
            ];
            for neighboring_face in neighboring_faces {
                if let Some(neigh_pos) = LightSolver::get_neighbor_pos(global_pos, &neighboring_face.0) {
                    let neigh_block_lid = world.get_block(&neigh_pos);
                    if let Some(neigh_block) = blocks_loader.loaded_blocks.get(neigh_block_lid as usize) {
                        //TODO soft lighting
                        if !neigh_block.mesh.is_cube() {
                            let lr: f32 = (world.get_light_level(&neigh_pos, R_CHANNEL) as f32) / 15.0f32;
                            let lg: f32 = (world.get_light_level(&neigh_pos, G_CHANNEL) as f32) / 15.0f32;
                            let lb: f32 = (world.get_light_level(&neigh_pos, B_CHANNEL) as f32) / 15.0f32;
                            let ls: f32 = (world.get_light_level(&neigh_pos, S_CHANNEL) as f32) / 15.0f32;
                            render_face(&neighboring_face.1, vertices, indices, pos, &[lr, lg, lb, ls]);
                        }
                    } else {
                        return Err(Box::new(BlockUsingError::BlockNotFoundError()));
                    }
                } else {
                    //TO MAKE EASIER WORLD BORDER RENDERING NOT REND THIS FACE ALSO CHANGE IN CUSTOM MESH
                    render_face(&neighboring_face.1, vertices, indices, pos, &[0.0f32, 0.0f32, 0.0f32, 1.0f32]); //default light level out of the world
                }
            }
        }
        Custom(custom_mesh) => {
            if custom_mesh.faces.len() != 0 {
                let mut rend = false;
                for neighbor in NEIGHBORHOOD {
                    if let Some(neighbor_block_lid) = get_neighbor_block(world, global_pos, &neighbor) {
                        if let Some(neighbor_block) = blocks_loader.loaded_blocks.get(neighbor_block_lid as usize) {
                            if !neighbor_block.mesh.is_cube() {
                                rend = true;
                                break;
                            }
                        } else {
                            return Err(Box::new(BlockUsingError::BlockNotFoundError()));
                        }
                    } else {
                        //TO MAKE EASIER WORLD BORDER RENDERING SET REND TO FALSE ALSO CHANGE CUBE RENDER
                        rend = true;
                        break;
                    }
                }
                if rend {
                    for face in &custom_mesh.faces {
                        let lr: f32 = (world.get_light_level(global_pos, R_CHANNEL) as f32) / 15.0f32;
                        let lg: f32 = (world.get_light_level(global_pos, G_CHANNEL) as f32) / 15.0f32;
                        let lb: f32 = (world.get_light_level(global_pos, B_CHANNEL) as f32) / 15.0f32;
                        let ls: f32 = (world.get_light_level(global_pos, S_CHANNEL) as f32) / 15.0f32;
                        render_face(&face, vertices, indices, pos, &[lr, lg, lb, ls]);
                    }
                }
            }
        }
    }
    return Ok(());
}