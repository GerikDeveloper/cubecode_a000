use std::cell::{Cell, RefCell};
use std::ops::Deref;
use std::ptr;
use thiserror::Error;
use crate::render::block_renderer;
use crate::render::blocks_loader::{AIR_BLOCK_ID, BlocksLoader, BlockUsingError, UNKNOWN_BLOCK_ID};
use crate::render::buffer::Buffer;
use crate::render::light::light_map::LightMap;
use crate::render::shader_program::ShaderProgram;
use crate::render::types::{Mat4f, Vec2ub, Vec3ub, LightedTexVertex};
use crate::render::vertex_array::VertexArray;
use crate::set_attribute;
use crate::world::World;

#[derive(Error, Debug)]
pub enum ChunkLoadingError {
    #[error("Id (b or l) byte of u16 not found")]
    IdConstructError(),
}

//TODO REPLACE CHUNKS WITH SUBCHUNKS
pub struct SubChunk {
    //TODO MB ANOTHER ARRAYS FOR CUSTOM BLOCKS
    pub data: RefCell<[[[u16; 16]; 16]; 16]>,
    pub light_map: RefCell<LightMap>,
    pub is_changed: Cell<bool>,
    pub vert_buf: RefCell<Buffer>,
    pub ind_buf: RefCell<Buffer>,
    pub ind_cnt: Cell<i32>,
    pub vert_array: RefCell<VertexArray>,
}

impl SubChunk {
    pub fn new(data: [[[u16; 16]; 16]; 16]) -> SubChunk {
        unsafe {
            return SubChunk {data: RefCell::new(data), light_map: RefCell::from(LightMap::new()), is_changed: Cell::new(true), vert_buf: RefCell::new(Buffer::new(gl::ARRAY_BUFFER)), ind_buf: RefCell::new(Buffer::new(gl::ELEMENT_ARRAY_BUFFER)), ind_cnt: Cell::new(0), vert_array: RefCell::new(VertexArray::new()), };
        }
    }

    pub fn render(&self, world: &World, blocks_loader: &BlocksLoader, subchunk_pos: &Vec3ub) -> Result<(), Vec<Box<dyn std::error::Error>>> {
        let mut errors: Vec<Box<dyn std::error::Error>> = Vec::new();
        if self.is_changed.get() {
            let mut vertices: Vec<LightedTexVertex> = Vec::new();
            let mut indices: Vec<i32> = Vec::new();
            for block_pos in 0..0x1000 {
                let block_pos_x = (block_pos & 0x0F) as u8;
                let block_pos_z = ((block_pos >> 4) & 0x0F) as u8;
                let block_pos_y = (block_pos >> 8) as u8;
                let block_lid: u16 = self.data.borrow()[block_pos_y as usize][block_pos_z as usize][block_pos_x as usize];
                //FAILED TO RENDER BLOCK
                match block_renderer::render_block(world, blocks_loader, block_lid, &mut vertices, &mut indices, subchunk_pos, &[block_pos_x, block_pos_y, block_pos_z]) {
                    Ok(_) => {}
                    Err(error) => {
                        errors.push(error);
                    }
                }
            }

            unsafe {
                self.vert_array.replace(VertexArray::new());
                self.vert_array.borrow().bind();

                self.vert_buf.replace(Buffer::new(gl::ARRAY_BUFFER));
                self.vert_buf.borrow().set_data(vertices.as_slice(), gl::STATIC_DRAW);

                match blocks_loader.meshes_loader.faces_loader.shader_program.get_attrib_location("pos") {
                    Ok(pos_attrib) => {

                        set_attribute!(self.vert_array.borrow(), pos_attrib, LightedTexVertex::0);

                        match blocks_loader.meshes_loader.faces_loader.shader_program.get_attrib_location("tex") {
                            Ok(tex_attrib) => {

                                set_attribute!(self.vert_array.borrow(), tex_attrib, LightedTexVertex::1);

                                match blocks_loader.meshes_loader.faces_loader.shader_program.get_attrib_location("light") {
                                    Ok(light_attrib) => {

                                        set_attribute!(self.vert_array.borrow(), light_attrib, LightedTexVertex::2);

                                        self.ind_buf.replace(Buffer::new(gl::ELEMENT_ARRAY_BUFFER));
                                        self.ind_buf.borrow().set_data(indices.as_slice(), gl::STATIC_DRAW);
                                    }
                                    Err(error) => {
                                        errors.push(Box::new(error));
                                    }
                                }
                            }
                            Err(error) => {
                                errors.push(Box::new(error));
                            }
                        }
                    }
                    Err(error) => {
                        errors.push(Box::new(error));
                    }
                }
            }

            self.ind_cnt.set(indices.len() as i32);
            self.is_changed.set(false);
        }

        return if errors.len() == 0 {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn draw(&self, shader_program: &ShaderProgram, pos: &Vec3ub) {
        if self.ind_cnt.get() != 0 {
            unsafe {
                if let Ok(_) = shader_program.set_uniform_mat4f("modelMat", &Mat4f::get_subchunk_model_mat(pos)) {
                    shader_program.apply();
                    self.vert_array.borrow().bind();
                    gl::DrawElements(gl::TRIANGLES, self.ind_cnt.get(), gl::UNSIGNED_INT, ptr::null());
                }
            }
        }
    }
}

pub struct Chunk {
    pub subchunks: [SubChunk; 16],
}

impl Chunk {
    pub fn new<T: ChunkGenerator>(generator: &T, blocks_loader: &BlocksLoader) -> Result<Self, Box<dyn std::error::Error>> {
        generator.get_chunk(blocks_loader)
    }

    pub fn store(&self, blocks_loader: &BlocksLoader) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut store_data: Vec<u8> = Vec::new();
        for mut subchunk in &self.subchunks {
            for mut plane_data in subchunk.data.borrow().deref() {
                for mut line_data in plane_data {
                    for block_data in line_data {
                        if let Some(block) = blocks_loader.loaded_blocks.get(*block_data as usize) {
                            store_data.push((block.id >> 8) as u8);
                            store_data.push(block.id as u8);
                        } else {
                            return Err(Box::new(BlockUsingError::BlockNotFoundError()));
                        }
                    }
                }
            }
        }
        return Ok( store_data );
    }

    pub fn load(blocks_loader: &BlocksLoader, data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        return if let Some(unknown_block) = blocks_loader.blocks_ids.get(&UNKNOWN_BLOCK_ID) {
            let mut chunk_data: [[[[u16; 16]; 16]; 16]; 16] = [[[[unknown_block.lid; 16]; 16]; 16]; 16];
            for subchunk_pos in 0..16u8 {
                for plane_pos in 0..16u8 {
                    for line_pos in 0..16u8 {
                        for block_pos in 0..16u8 {
                            let block_data_pos = ((((subchunk_pos as u16) << 12) | ((plane_pos as u16) << 8) | ((line_pos as u16) << 4) | (block_pos as u16)) as usize) << 1;
                            if let Some(b_block_data) = data.get(block_data_pos) {
                                if let Some(l_block_data) = data.get(block_data_pos + 1) {
                                    let block_id = (((*b_block_data as u16) << 8) | (*l_block_data as u16));
                                    if let Some(block) = blocks_loader.blocks_ids.get(&block_id) {
                                        chunk_data[subchunk_pos as usize][plane_pos as usize][line_pos as usize][block_pos as usize] = block.lid;
                                    } else {
                                        return Err(Box::new(BlockUsingError::BlockNotFoundError()));
                                    }
                                } else {
                                    return Err(Box::new(ChunkLoadingError::IdConstructError()));
                                }
                            } else {
                                return Err(Box::new(ChunkLoadingError::IdConstructError()));
                            }
                        }
                    }
                }
            }
            Ok(Chunk {
                subchunks: [
                    SubChunk::new(chunk_data[0 ]),
                    SubChunk::new(chunk_data[1 ]),
                    SubChunk::new(chunk_data[2 ]),
                    SubChunk::new(chunk_data[3 ]),
                    SubChunk::new(chunk_data[4 ]),
                    SubChunk::new(chunk_data[5 ]),
                    SubChunk::new(chunk_data[6 ]),
                    SubChunk::new(chunk_data[7 ]),
                    SubChunk::new(chunk_data[8 ]),
                    SubChunk::new(chunk_data[9 ]),
                    SubChunk::new(chunk_data[10]),
                    SubChunk::new(chunk_data[11]),
                    SubChunk::new(chunk_data[12]),
                    SubChunk::new(chunk_data[13]),
                    SubChunk::new(chunk_data[14]),
                    SubChunk::new(chunk_data[15]),
                ],
            })
        } else {
            Err(Box::new(BlockUsingError::BlockNotFoundError()))
        }
    }

    pub fn set_data(&self, chunk: Chunk) {
        for subchunk_pos in 0..16u8 {
            self.subchunks[subchunk_pos as usize].data.replace_with(|_| *chunk.subchunks[subchunk_pos as usize].data.borrow());
            self.subchunks[subchunk_pos as usize].is_changed.set(true);
        }
    }

    pub fn render(&self, world: &World, blocks_loader: &BlocksLoader, chunk_pos: &Vec2ub) -> Result<(), Vec<Box<dyn std::error::Error>>> {
        let mut errors: Vec<Box<dyn std::error::Error>> = Vec::new();
        for subchunk_pos in 0..16u8 {
            match self.subchunks[subchunk_pos as usize].render(world, blocks_loader, &[subchunk_pos, chunk_pos[0], chunk_pos[1]]) {
                Ok(_) => {}
                Err(new_errors) => {
                    for error in new_errors {
                        errors.push(error);
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

    pub fn draw(&self, shader_program: &ShaderProgram, pos: &Vec3ub) {
        for chunk_pos in 0..16u8 {
            self.subchunks[chunk_pos as usize].draw(shader_program, &[pos[0], pos[1] + (chunk_pos << 4), pos[2]]);
        }
    }
}

pub trait ChunkGenerator {
    fn get_chunk(&self, blocks_loader: &BlocksLoader) -> Result<Chunk, Box<dyn std::error::Error>>;
}



pub struct LayerChunkGenerator {
    pub layers: [[u16; 16]; 16],
}

impl LayerChunkGenerator {

    pub fn new() -> Self {
        Self { layers: [[AIR_BLOCK_ID; 16]; 16] }
    }

    pub fn from_bottom_layers(bottom_layers: &[u16]) -> Self {
        let mut layers: [[u16; 16]; 16] = [[AIR_BLOCK_ID; 16]; 16];
        let top_pos = {
            if bottom_layers.len() < 256 {
                bottom_layers.len()
            } else {
                256
            }
        };
        for layer_pos in 0..top_pos {
            layers[layer_pos >> 4][layer_pos & 0x0F] = bottom_layers[layer_pos];
        }

        Self { layers }
    }

    pub fn from_top_layers(top_layers: &[u16]) -> Self {
        let mut layers: [[u16; 16]; 16] = [[AIR_BLOCK_ID; 16]; 16];
        let bottom_pos = {
            if top_layers.len() < 256 {
                256 - top_layers.len()
            } else {
                0
            }
        };
        for layer_pos in bottom_pos..256 {
            layers[layer_pos >> 4][layer_pos & 0x0F] = top_layers[255 - layer_pos];
        }

        Self { layers }
    }
}

impl ChunkGenerator for LayerChunkGenerator {
    fn get_chunk(&self, blocks_loader: &BlocksLoader) -> Result<Chunk, Box<dyn std::error::Error>> {
        return if let Some(unknown_block) = blocks_loader.blocks_ids.get(&UNKNOWN_BLOCK_ID) {
            let mut data: [[[[u16; 16]; 16]; 16]; 16] = [[[[unknown_block.lid; 16]; 16]; 16]; 16];
            for subchunk_pos in 0..16u8 {
                for layer_pos in 0..16u8 {
                    if let Some(block) = blocks_loader.blocks_ids.get(&self.layers[subchunk_pos as usize][layer_pos as usize]) {
                        data[subchunk_pos as usize][layer_pos as usize] = [[block.lid; 16]; 16];
                    } else {
                        return Err(Box::new(BlockUsingError::BlockNotFoundError()));
                    }
                }
            }
            Ok(Chunk {
                subchunks: [
                    SubChunk::new(data[0 ]),
                    SubChunk::new(data[1 ]),
                    SubChunk::new(data[2 ]),
                    SubChunk::new(data[3 ]),
                    SubChunk::new(data[4 ]),
                    SubChunk::new(data[5 ]),
                    SubChunk::new(data[6 ]),
                    SubChunk::new(data[7 ]),
                    SubChunk::new(data[8 ]),
                    SubChunk::new(data[9 ]),
                    SubChunk::new(data[10]),
                    SubChunk::new(data[11]),
                    SubChunk::new(data[12]),
                    SubChunk::new(data[13]),
                    SubChunk::new(data[14]),
                    SubChunk::new(data[15]),
                ],
            })
        } else {
            Err(Box::new(BlockUsingError::BlockNotFoundError()))
        }
    }
}