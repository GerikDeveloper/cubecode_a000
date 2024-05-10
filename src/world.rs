use std::fs::File;
use std::io::{Read, Write};
use std::ops::{Deref, DerefMut};
use thiserror::Error;
use crate::chunk::{Chunk, ChunkGenerator};
use crate::render::blocks_loader::{AIR_BLOCK_ID, BlocksLoader};
use crate::render::types::{Vec3f, Vec3i, Vec3s, Vec3ub};

const CHUNK_SIZE: usize = 16 * 16 * 16 * 16 * 2;

#[derive(Error, Debug)]
pub enum WorldCreationError {
    #[error("Vector sizing error")]
    VectorSizingError(),
}

#[derive(Error, Debug)]
pub enum WorldLoadingError {
    #[error("Invalid chunk size")]
    InvalidChunkSizeError(),
}

pub struct World {
    pub chunks: [[Box<Chunk>; 16]; 16],
}

impl World {
    pub fn new<T: ChunkGenerator>(chunk_generator: &T, blocks_loader: &BlocksLoader) -> Result<Self, Box<dyn std::error::Error>> {
        let mut chunks_plane: Vec<[Box<Chunk>; 16]> = Vec::new();
        for _chunk_line_pos in 0..16u8 {
            let mut chunk_line: Vec<Box<Chunk>> = Vec::new();
            for _chunk_pos in 0..16u8 {
                chunk_line.push(Box::new(Chunk::new(chunk_generator, blocks_loader)?));
            }
            let chunk_line_data_res: Result<[Box<Chunk>; 16], Vec<Box<Chunk>>> = chunk_line.try_into();
            if let Ok(chunk_line_data) = chunk_line_data_res {
                chunks_plane.push(chunk_line_data);
            } else {
                return Err(Box::new(WorldCreationError::VectorSizingError()));
            }
        }
        let chunks_res: Result<[[Box<Chunk>; 16]; 16], Vec<[Box<Chunk>; 16]>> = chunks_plane.try_into();
        return if let Ok(chunks) = chunks_res {
            Ok(Self { chunks })
        } else {
            Err(Box::new(WorldCreationError::VectorSizingError()))
        }
    }

    pub fn get_block(&self, pos: &Vec3ub) -> u16 {
        let subchunk_pos: Vec3ub = [pos[0] >> 4, pos[1] >> 4, pos[2] >> 4];
        let block_pos: Vec3ub = [pos[0] & 0x0F, pos[1] & 0x0F, pos[2] & 0x0F];
        self.chunks[subchunk_pos[0] as usize][subchunk_pos[2] as usize].subchunks[subchunk_pos[1] as usize].data.borrow()[block_pos[1] as usize][block_pos[2] as usize][block_pos[0] as usize]
    }

    pub fn set_block(&self, pos: &Vec3ub, block_lid: u16) {
        let subchunk_pos: Vec3ub = [pos[0] >> 4, pos[1] >> 4, pos[2] >> 4];
        let block_pos: Vec3ub = [pos[0] & 0x0F, pos[1] & 0x0F, pos[2] & 0x0F];
        self.chunks[subchunk_pos[0] as usize][subchunk_pos[2] as usize].subchunks[subchunk_pos[1] as usize].data.borrow_mut()[block_pos[1] as usize][block_pos[2] as usize][block_pos[0] as usize] = block_lid;
        self.chunks[subchunk_pos[0] as usize][subchunk_pos[2] as usize].subchunks[subchunk_pos[1] as usize].is_changed.set(true);

        if block_pos[0] == 0x0F && subchunk_pos[0] < 0x0F {self.chunks[(subchunk_pos[0] + 1) as usize][subchunk_pos[2] as usize].subchunks[subchunk_pos[1] as usize].is_changed.set(true);}
        if block_pos[1] == 0x0F && subchunk_pos[1] < 0x0F {self.chunks[subchunk_pos[0] as usize][subchunk_pos[2] as usize].subchunks[(subchunk_pos[1] + 1) as usize].is_changed.set(true);}
        if block_pos[2] == 0x0F && subchunk_pos[2] < 0x0F {self.chunks[subchunk_pos[0] as usize][(subchunk_pos[2] + 1) as usize].subchunks[subchunk_pos[1] as usize].is_changed.set(true);}

        if block_pos[0] == 0x00 && subchunk_pos[0] > 0x00 {self.chunks[(subchunk_pos[0] - 1) as usize][subchunk_pos[2] as usize].subchunks[subchunk_pos[1] as usize].is_changed.set(true);}
        if block_pos[1] == 0x00 && subchunk_pos[1] > 0x00 {self.chunks[subchunk_pos[0] as usize][subchunk_pos[2] as usize].subchunks[(subchunk_pos[1] - 1) as usize].is_changed.set(true);}
        if block_pos[2] == 0x00 && subchunk_pos[2] > 0x00 {self.chunks[subchunk_pos[0] as usize][(subchunk_pos[2] - 1) as usize].subchunks[subchunk_pos[1] as usize].is_changed.set(true);}
    }

    pub fn ray_get(&self, pos: &Vec3f, dir: &Vec3f, max_dist: f32, end: &mut Vec3f, norm: &mut Vec3f, iend: &mut Vec3ub) -> Option<u16> {
        let mut pdist: f32 = 0.0; //passed dist

        let mut ipos: Vec3i = [(pos[0] as i32), (pos[1] as i32), (pos[2] as i32)];

        let stepx: f32 = {if dir[0] > 0.0f32 { 1.0 } else { -1.0 }};
        let stepy: f32 = {if dir[1] > 0.0f32 { 1.0 } else { -1.0 }};
        let stepz: f32 = {if dir[2] > 0.0f32 { 1.0 } else { -1.0 }};

        /*
        indicates how far along the ray we must move (in units of pdist) for the horizontal component of such a movement to equal the width of a voxel.
         */
        let tdx: f32 = {if dir[0] == 0.0f32 { f32::INFINITY } else { f32::abs(1.0f32 / dir[0]) }};
        let tdy: f32 = {if dir[1] == 0.0f32 { f32::INFINITY } else { f32::abs(1.0f32 / dir[1]) }};
        let tdz: f32 = {if dir[2] == 0.0f32 { f32::INFINITY } else { f32::abs(1.0f32 / dir[2]) }};

        let xdist: f32 = {if stepx > 0.0f32 { (ipos[0] as f32) + 1.0f32 - pos[0] } else { pos[0] - (ipos[0] as f32) }};
        let ydist: f32 = {if stepy > 0.0f32 { (ipos[1] as f32) + 1.0f32 - pos[1] } else { pos[1] - (ipos[1] as f32) }};
        let zdist: f32 = {if stepz > 0.0f32 { (ipos[2] as f32) + 1.0f32 - pos[2] } else { pos[2] - (ipos[2] as f32) }};


        /*
        we determine the value of pdist at which the ray crosses the first vertical voxel boundary and store it in variable tMaxX.
         */
        let mut txmax: f32 = {if tdx < f32::INFINITY { tdx * xdist } else { f32::INFINITY }}; //smallest positive pdist such that pos[0] + pdist * dir[0] is an int
        let mut tymax: f32 = {if tdy < f32::INFINITY { tdy * ydist } else { f32::INFINITY }};
        let mut tzmax: f32 = {if tdz < f32::INFINITY { tdz * zdist } else { f32::INFINITY }};

        let mut stepind: i8 = -1;

        while pdist <= max_dist {
            if ipos[0] <= 0xFF && ipos[0] >= 0x00 &&
                ipos[1] <= 0xFF && ipos[1] >= 0x00 &&
                ipos[2] <= 0xFF && ipos[2] >= 0x00 {
                let block = self.get_block(&[(ipos[0] as u8), (ipos[1] as u8), (ipos[2] as u8)]);
                if block != AIR_BLOCK_ID { //TODO REWRITE for not cube blocks
                    end[0] = pos[0] + pdist * dir[0];
                    end[1] = pos[1] + pdist * dir[1];
                    end[2] = pos[2] + pdist * dir[2];

                    iend[0] = ipos[0] as u8;
                    iend[1] = ipos[1] as u8;
                    iend[2] = ipos[2] as u8;

                    norm[0] = 0.0f32;
                    norm[1] = 0.0f32;
                    norm[2] = 0.0f32;
                    match stepind {
                        0 => norm[0] = -stepx,
                        1 => norm[1] = -stepy,
                        2 => norm[2] = -stepz,
                        _ => {}
                    }
                    return Some(block);
                }
                if txmax < tymax {
                    if txmax < tzmax {
                        ipos[0] += stepx as i32;
                        pdist = txmax;
                        txmax += tdx;
                        stepind = 0;
                    } else {
                        ipos[2] += stepz as i32;
                        pdist = tzmax;
                        tzmax += tdz;
                        stepind = 2;
                    }
                } else {
                    if tymax < tzmax {
                        ipos[1] += stepy as i32;
                        pdist = tymax;
                        tymax += tdy;
                        stepind = 1;
                    } else {
                        ipos[2] += stepz as i32;
                        pdist = tzmax;
                        tzmax += tdz;
                        stepind = 2;
                    }
                }
            } else {
                break;
            }
        }

        iend[0] = ipos[0] as u8;
        iend[1] = ipos[1] as u8;
        iend[2] = ipos[2] as u8;

        end[0] = pos[0] + (pdist * dir[0]);
        end[1] = pos[1] + (pdist * dir[1]);
        end[2] = pos[2] + (pdist * dir[2]);

        norm[0] = 0.0f32;
        norm[1] = 0.0f32;
        norm[2] = 0.0f32;
        return None;
    }

    pub fn get_light_level(&self, pos: &Vec3ub, channel: u8) -> u8 {
        let subchunk_pos: Vec3ub = [pos[0] >> 4, pos[1] >> 4, pos[2] >> 4];
        let block_pos: Vec3ub = [pos[0] & 0x0F, pos[1] & 0x0F, pos[2] & 0x0F];
        return self.chunks[subchunk_pos[0] as usize][subchunk_pos[2] as usize].subchunks[subchunk_pos[1] as usize].light_map.borrow().get(&block_pos, channel);
    }

    pub fn set_light_level(&self, pos: &Vec3ub, channel: u8, level: u8) {
        let subchunk_pos: Vec3ub = [pos[0] >> 4, pos[1] >> 4, pos[2] >> 4];
        let block_pos: Vec3ub = [pos[0] & 0x0F, pos[1] & 0x0F, pos[2] & 0x0F];
        self.chunks[subchunk_pos[0] as usize][subchunk_pos[2] as usize].subchunks[subchunk_pos[1] as usize].light_map.borrow().set(&block_pos, channel, level);
        self.chunks[subchunk_pos[0] as usize][subchunk_pos[2] as usize].subchunks[subchunk_pos[1] as usize].is_changed.set(true);
    }

    pub fn render(&self, blocks_loader: &BlocksLoader) -> Result<(), Vec<Box<dyn std::error::Error>>> {
        let mut errors: Vec<Box<dyn std::error::Error>> = Vec::new();
        for line_pos in 0..16u8 {
            for chunk_pos in 0..16u8 {
                match self.chunks[line_pos as usize][chunk_pos as usize].render(self, blocks_loader, &[line_pos, chunk_pos]) {
                    Ok(_) => {}
                    Err(new_errors) => {
                        for error in new_errors {
                            errors.push(error);
                        }
                    }
                }
            }
        }
        return if errors.len() == 0 {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn draw(&self, blocks_loader: &BlocksLoader) {
        unsafe {
            blocks_loader.meshes_loader.faces_loader.atlas.activate(gl::TEXTURE0);
        }
        for line_pos in 0..16u8 {
            for chunk_pos in 0..16u8 {
                self.chunks[line_pos as usize][chunk_pos as usize].draw(&blocks_loader.meshes_loader.faces_loader.shader_program, &[(line_pos << 4), 0, (chunk_pos << 4)]);
            }
        }
    }

    pub fn store(&self, blocks_loader: &BlocksLoader, file_name: String) -> Result<(), Box<dyn std::error::Error>> {
        return match File::create(file_name) {
            Err(error) => {
                Err(Box::new(error))
            }
            Ok(mut file) => {
                for line_pos in 0..16u8 {
                    for chunk_pos in 0..16u8 {
                        match self.chunks[line_pos as usize][chunk_pos as usize].store(blocks_loader) {
                            Err(error) => {
                                return Err(error);
                            }
                            Ok(chunk_data) => {
                                if let Err(error) = file.write_all(chunk_data.as_slice()) {
                                    return Err(Box::new(error));
                                }
                            }
                        }
                    }
                }
                Ok(())
            }
        }
    }

    pub fn load(&self, blocks_loader: &BlocksLoader, file_name: String) -> Result<(), Box<dyn std::error::Error>> {
        match File::open(file_name) {
            Err(error) => {
                return Err(Box::new(error));
            }
            Ok(mut file) => {
                for line_pos in 0..16u8 {
                    for chunk_pos in 0..16u8 {
                        let mut data: Box<[u8; CHUNK_SIZE]> = Box::new([0u8; CHUNK_SIZE]);
                        match file.read(data.deref_mut()) {
                            Err(error) => {
                                return Err(Box::new(error));
                            }
                            Ok(size) => {
                                if size == CHUNK_SIZE {
                                    match Chunk::load(blocks_loader, data.deref_mut()) {
                                        Err(error) => {
                                            return Err(error);
                                        }
                                        Ok(chunk) => {
                                            self.chunks[line_pos as usize][chunk_pos as usize].set_data(chunk);
                                        }
                                    }
                                } else {
                                    return Err(Box::new(WorldLoadingError::InvalidChunkSizeError()));
                                }
                            }
                        }
                    }
                }
                return Ok(());
            }
        }
    }

}


//TODO NEXT: LOAD AND STORE WORLD SAVING WORLD IN DROP