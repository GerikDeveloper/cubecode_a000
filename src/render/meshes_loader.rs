use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::rc::Rc;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use crate::render::faces_loader::{Face, FacesLoader};
use crate::render::meshes_loader::Mesh::Cube;

#[derive(Error, Debug)]
pub enum MeshesLoadingError {
    #[error("Deserialization failed")]
    DeserializationError(),
    #[error("Mesh type has unknown type")]
    UnknownTypeError(),
    #[error("Unknown face error")]
    UnknownFaceError(),
    #[error("Redefinition error")]
    RedefinitionError(),
}

//TODO MB CHECK BLOCK BORDER

#[derive(Serialize, Deserialize)]
struct CubeMeshData {
    pub(crate) top: String,
    pub(crate) bottom: String,
    pub(crate) front: String,
    pub(crate) back: String,
    pub(crate) right: String,
    pub(crate) left: String,
}

#[derive(Serialize, Deserialize)]
struct CustomMeshData {
    pub(crate) faces: Vec<String>,
}

//TODO MB MULTI THREAD ARC

//ANALYZE FACES AND MB CREATE SHADERS RCS

//SINGLE THREAD
pub(crate) struct CubeMesh {
    pub(crate) top: Rc<Face>,
    pub(crate) bottom: Rc<Face>,
    pub(crate) front: Rc<Face>,
    pub(crate) back: Rc<Face>,
    pub(crate) right: Rc<Face>,
    pub(crate) left: Rc<Face>,
}

pub(crate) struct CustomMesh {
    pub(crate) faces: Vec<Rc<Face>>,
}

pub enum Mesh {
    Cube(CubeMesh),
    Custom(CustomMesh),
}

impl Mesh {
    pub fn is_cube(&self) -> bool {
        return if let Cube(_) = self {
            true
        } else {
            false
        }
    }
}

pub struct MeshesLoader {
    pub(crate) meshes: HashMap<String, Rc<Mesh>>,
    pub faces_loader: FacesLoader,
}

impl MeshesLoader {
    pub fn load(meshes_path: &Path, faces_loader: FacesLoader) -> Result<Self, Box<dyn std::error::Error>> {
        let mut meshes: HashMap<String, Rc<Mesh>> = HashMap::new();
        let meshes_data = fs::read_to_string(meshes_path)?;
        let serialized: serde_json::Value = serde_json::from_str(&meshes_data)?;
        if let Some(meshes_values) = serialized.as_array() {
            for mesh_value in meshes_values {
                if let Some(name_value) = mesh_value.get("name") {
                    if let Some(mesh_type_value) = mesh_value.get("mesh_type") {
                        if let Some(name_str) = name_value.as_str() {
                            if let Some(mesh_type_str) = mesh_type_value.as_str() {
                                if let Some(mesh_value) = mesh_value.get("mesh") {
                                    match mesh_type_str {
                                        "cube" => {
                                            let mesh: CubeMeshData = CubeMeshData::deserialize(mesh_value)?;
                                            if let Some(top_face) = faces_loader.faces.get(&mesh.top) {
                                                if let Some(bottom_face) = faces_loader.faces.get(&mesh.bottom) {
                                                    if let Some(front_face) = faces_loader.faces.get(&mesh.front) {
                                                        if let Some(back_face) = faces_loader.faces.get(&mesh.back) {
                                                            if let Some(right_face) = faces_loader.faces.get(&mesh.right) {
                                                                if let Some(left_face) = faces_loader.faces.get(&mesh.left) {
                                                                    if let Some(_) = meshes.insert(String::from(name_str), Rc::new(Cube(CubeMesh {
                                                                        top: top_face.clone(),
                                                                        bottom: bottom_face.clone(),
                                                                        front: front_face.clone(),
                                                                        back: back_face.clone(),
                                                                        right: right_face.clone(),
                                                                        left: left_face.clone(),
                                                                    }))) {
                                                                        return Err(Box::new(MeshesLoadingError::RedefinitionError()));
                                                                    }
                                                                } else {
                                                                    return Err(Box::new(MeshesLoadingError::UnknownFaceError()));
                                                                }
                                                            } else {
                                                                return Err(Box::new(MeshesLoadingError::UnknownFaceError()));
                                                            }
                                                        } else {
                                                            return Err(Box::new(MeshesLoadingError::UnknownFaceError()));
                                                        }
                                                    } else {
                                                        return Err(Box::new(MeshesLoadingError::UnknownFaceError()));
                                                    }
                                                } else {
                                                    return Err(Box::new(MeshesLoadingError::UnknownFaceError()));
                                                }
                                            } else {
                                                return Err(Box::new(MeshesLoadingError::UnknownFaceError()));
                                            }
                                        }
                                        "custom" => {
                                            let mesh: CustomMeshData = CustomMeshData::deserialize(mesh_value)?;
                                            let mut faces: Vec<Rc<Face>> = Vec::new();
                                            for face in &mesh.faces {
                                                if let Some(face) = faces_loader.faces.get(face) {
                                                    faces.push(face.clone());
                                                } else {
                                                    return Err(Box::new(MeshesLoadingError::UnknownFaceError()));
                                                }
                                            }
                                            if let Some(_) = meshes.insert(String::from(name_str), Rc::new(Mesh::Custom(CustomMesh { faces }))) {
                                                return Err(Box::new(MeshesLoadingError::RedefinitionError()));
                                            }
                                        }
                                        _ => {
                                            return Err(Box::new(MeshesLoadingError::UnknownTypeError()));
                                        }
                                    }
                                } else {
                                    return Err(Box::new(MeshesLoadingError::DeserializationError()));
                                }
                            } else {
                                return Err(Box::new(MeshesLoadingError::DeserializationError()));
                            }
                        } else {
                            return Err(Box::new(MeshesLoadingError::DeserializationError()));
                        }
                    } else {
                        return Err(Box::new(MeshesLoadingError::DeserializationError()));
                    }
                } else {
                    return Err(Box::new(MeshesLoadingError::DeserializationError()));
                }
            }
        } else {
            return Err(Box::new(MeshesLoadingError::DeserializationError()));
        }
        Ok(Self {
            meshes,
            faces_loader,
        })
    }
}