//TODO Loading behavior
//TODO struct BlockBehavior

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::rc::Rc;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use crate::render::meshes_loader::{Mesh, MeshesLoader};


#[derive(Error, Debug)]
pub enum BlocksLoadingError {
    #[error("Deserialization failed")]
    DeserializationError(),
    #[error("Redefinition error")]
    RedefinitionError(),
    #[error("Default block not found error")]
    DefaultBlockNotFoundError(),
    #[error("Unknown mesh error")]
    UnknownMeshError(),
}

#[derive(Error, Debug)]
pub enum BlockUsingError {
    #[error("Block not found")]
    BlockNotFoundError(),
}


pub const AIR_BLOCK_ID: u16 = 0;
pub const UNKNOWN_BLOCK_ID: u16 = 1;
pub const DIRT_BLOCK_ID: u16 = 2;
pub const GRASS_BLOCK_ID: u16 = 3;
pub const BEDROCK_BLOCK_ID: u16 = 4;

//TODO ARRAY OF BLOCKS WITHOUT DEFINING IDS IN JSON

const DEFAULT_BLOCKS: &[(u16, &str)] = &[
    (AIR_BLOCK_ID,                  "air"),
    (UNKNOWN_BLOCK_ID,              "unknown"),
    (DIRT_BLOCK_ID,                 "dirt"),
    (GRASS_BLOCK_ID,                "grass"),
    (BEDROCK_BLOCK_ID,              "bedrock"),
];

#[derive(Clone, Serialize, Deserialize)]
pub struct BlockData {
    pub id: u16,
    pub name: String,
    pub light_r: u8,
    pub light_g: u8,
    pub light_b: u8,
    pub mesh: String,
    //TODO behavior
}

pub struct Block {
    //TODO without lid
    pub lid: u16,
    pub id: u16,
    pub name: String,
    //TODO MB rename to lum, lumen, illumination or something
    pub light_r: u8,
    pub light_g: u8,
    pub light_b: u8,
    pub mesh: Rc<Mesh>,
}

pub struct BlocksLoader {
    pub loaded_blocks: Vec<Rc<Block>>,
    pub blocks_names: HashMap<String, Rc<Block>>,
    pub blocks_ids: HashMap<u16, Rc<Block>>,
    pub meshes_loader: MeshesLoader,
    pub unknown_block: Rc<Block>,
}

impl BlocksLoader {

    //TODO rewrite everywhere with this func
    pub fn get_block(&self, block_lid: u16) -> Rc<Block> {
        return if let Some(block) = self.loaded_blocks.get(block_lid as usize) {
            block.clone()
        } else {
            self.unknown_block.clone()
        }
    }

    pub fn load(blocks_path: &Path, meshes_loader: MeshesLoader) -> Result<Self, Box<dyn std::error::Error>> {
        let mut loaded_blocks: Vec<Rc<Block>> = Vec::new();
        let mut blocks_names: HashMap<String, Rc<Block>> = HashMap::new();
        let mut blocks_ids: HashMap<u16, Rc<Block>> = HashMap::new();
        let blocks_data = fs::read_to_string(blocks_path)?;
        let serialized: serde_json::Value = serde_json::from_str(&blocks_data)?;
        if let Some(blocks_values) = serialized.as_array() {
            for block_value in blocks_values {
                let block_data: BlockData = BlockData::deserialize(block_value)?;
                if let Some(mesh) = meshes_loader.meshes.get(&block_data.mesh) {
                    let block_name = block_data.name.clone();
                    let block_ref: Rc<Block> = Rc::new( Block{
                        lid: loaded_blocks.len() as u16,
                        id: block_data.id,
                        name: block_data.name,
                        light_r: block_data.light_r,
                        light_g: block_data.light_g,
                        light_b: block_data.light_b,
                        mesh: mesh.clone(),
                    } );
                    loaded_blocks.push(block_ref.clone());
                    if let Some(_) = blocks_names.insert(block_name, block_ref.clone()) {
                        return Err(Box::new(BlocksLoadingError::RedefinitionError()));
                    }
                    if let Some(_) = blocks_ids.insert(block_data.id, block_ref.clone()) {
                        return Err(Box::new(BlocksLoadingError::RedefinitionError()));
                    }
                } else {
                    return Err(Box::new(BlocksLoadingError::UnknownMeshError()));
                }
            }
            for default_block in DEFAULT_BLOCKS {
                if let Some(block) = blocks_names.get(default_block.1) {
                    if block.id != default_block.0 {
                        return Err(Box::new(BlocksLoadingError::DefaultBlockNotFoundError()));
                    }
                    if let Some(loaded_block) = loaded_blocks.get(block.lid as usize) {
                        if !Rc::ptr_eq(loaded_block, block) {
                            return Err(Box::new(BlocksLoadingError::DefaultBlockNotFoundError()));
                        }
                    }
                } else {
                    return Err(Box::new(BlocksLoadingError::DefaultBlockNotFoundError()));
                }
                if let Some(block) = blocks_ids.get(&default_block.0) {
                    if block.name != default_block.1 {
                        return Err(Box::new(BlocksLoadingError::DefaultBlockNotFoundError()));
                    }
                    if let Some(loaded_block) = loaded_blocks.get(block.lid as usize) {
                        if !Rc::ptr_eq(loaded_block, block) {
                            return Err(Box::new(BlocksLoadingError::DefaultBlockNotFoundError()));
                        }
                    }
                } else {
                    return Err(Box::new(BlocksLoadingError::DefaultBlockNotFoundError()));
                }
            }
        } else {
            return Err(Box::new(BlocksLoadingError::DeserializationError()));
        }
        return if let Some(block) = loaded_blocks.get(UNKNOWN_BLOCK_ID as usize) {
            let unknown_block = block.clone();
            Ok(Self { loaded_blocks, blocks_names, blocks_ids, meshes_loader, unknown_block })
        } else {
            Err(Box::new(BlocksLoadingError::DefaultBlockNotFoundError()))
        }
    }
}