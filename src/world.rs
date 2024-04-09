use std::fs::File;
use std::io::{Read, Write};
use std::ops::{Deref, DerefMut};
use thiserror::Error;
use crate::chunk::{Chunk, ChunkGenerator};
use crate::render::blocks_loader::BlocksLoader;
use crate::render::types::{Vec3ub};

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