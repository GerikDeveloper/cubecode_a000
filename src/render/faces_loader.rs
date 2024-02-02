use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::rc::Rc;
use crate::render::texture::Texture;
use crate::render::types::{Vertex};
use serde::{Serialize, Deserialize};
use thiserror::Error;
use crate::render::shader_program::ShaderProgram;

//TODO TEX LOADER MUL TEX COORDS TEX NAME IN FACE AND LOCAL TEX COORDS IN JSON
//TODO INTERPOLATION IN FACE BEHAVIOR

#[derive(Error, Debug)]
pub enum FacesLoadingError {
    #[error("Deserialization failed")]
    DeserializationError(),
    #[error("Redefinition error")]
    RedefinitionError(),
    #[error("Wrong indices count error")]
    WrongIndicesCountError(),
}


#[derive(Serialize, Deserialize)]
pub(crate) struct Face {
    pub(crate) name: String,
    pub(crate) vertices: Vec<Vertex>,
    pub(crate) indices: Vec<i32>,
}

pub struct FacesLoader {
    pub atlas: Texture,
    pub faces: HashMap<String, Rc<Face>>,
    pub shader_program: Rc<ShaderProgram>,
    //TODO SHADERS HASHMAP AND SHADER NAME IN JSON AND SHADERS CODE PATH IN JSON AND SHADERS CODES FILES .glsl
}

impl FacesLoader {
    pub fn load(atlas_path: &Path, faces_path: &Path, shader_program: Rc<ShaderProgram>) -> Result<Self, Box<dyn std::error::Error>> {
        unsafe {
            let mut atlas = Texture::new();
            atlas.load(atlas_path)?;
            shader_program.set_uniform_i32("tex", 0)?;
            let mut faces: HashMap<String, Rc<Face>> = HashMap::new();
            let faces_data = fs::read_to_string(faces_path)?;
            let serialized: serde_json::Value = serde_json::from_str(&faces_data)?;
            if let Some(faces_values) = serialized.as_array() {
                for face_value in faces_values {
                    let face_data: Face = Face::deserialize(face_value)?;
                    if (face_data.indices.len() % 3) == 0 {
                        if let Some(_) = faces.insert(face_data.name.clone(), Rc::new( Face {
                            name: face_data.name,
                            vertices: face_data.vertices,
                            indices: face_data.indices,
                        } )) {
                            return Err(Box::new(FacesLoadingError::RedefinitionError()));
                        }
                    } else {
                        return Err(Box::new(FacesLoadingError::WrongIndicesCountError()));
                    }
                }
            } else {
                return Err(Box::new(FacesLoadingError::DeserializationError()));
            }
            Ok(Self {
                atlas,
                faces,
                shader_program,
            })
        }
    }
}


