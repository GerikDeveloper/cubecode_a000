use std::path::Path;
use std::ptr;
use std::rc::Rc;
use glfw::ffi::KEY_ESCAPE;
use cubecode_a000::chunk::{Chunk, LayerChunkGenerator, SubChunk};
use cubecode_a000::input::keyboard::Keyboard;
use cubecode_a000::render::blocks_loader::{BEDROCK_BLOCK_ID, BlocksLoader, DIRT_BLOCK_ID, GRASS_BLOCK_ID, UNKNOWN_BLOCK_ID};
use cubecode_a000::render::buffer::Buffer;
use cubecode_a000::render::camera::Camera;
use cubecode_a000::render::faces_loader::FacesLoader;
use cubecode_a000::render::meshes_loader::MeshesLoader;
use cubecode_a000::render::shader::Shader;
use cubecode_a000::render::shader_program::ShaderProgram;
use cubecode_a000::render::types::{Mat4f, Vec3f, Vertex};
use cubecode_a000::render::vertex_array::VertexArray;
use cubecode_a000::set_attribute;
use cubecode_a000::window::Window;
use cubecode_a000::world::World;

const SCR_WIDTH: u32 = 800;
const SCR_HEIGHT: u32 = 600;
const TITLE: &str = "CubeCode >_";
const BLOCKS_ATLAS_PATH: &str = "assets/blocks_atlas.png";
const FACES_PATH: &str = "assets/faces.json";
const MESHES_PATH: &str = "assets/meshes.json";
const BLOCKS_PATH: &str = "assets/blocks.json";

fn get_blocks_loader(shader_program: Rc<ShaderProgram>) -> Result<BlocksLoader, Box<dyn std::error::Error>> {
    Ok(BlocksLoader::load(Path::new(BLOCKS_PATH), MeshesLoader::load(Path::new(MESHES_PATH), FacesLoader::load(Path::new(BLOCKS_ATLAS_PATH), Path::new(FACES_PATH), shader_program)?)?)?)
}

fn get_shader_program(vertex_src: &str, fragment_src: &str) -> Result<ShaderProgram, Box<dyn std::error::Error>> {
    unsafe {
        let vert_shader = Shader::new(vertex_src, gl::VERTEX_SHADER)?;
        let frag_shader = Shader::new(fragment_src, gl::FRAGMENT_SHADER)?;
        let shader_program = ShaderProgram::new(&[vert_shader, frag_shader])?;
        shader_program.set_uniform_i32("atlas", 0)?;
        return Ok(shader_program);
    }
}

pub const VERTEX_SHADER_SOURCE: &str = r#"
#version 330

in vec3 pos;
in vec2 tex;

out vec2 outTexCoord;

uniform mat4 viewMat;
uniform mat4 modelMat;

void main() {
    gl_Position = viewMat * modelMat * vec4(pos, 1.0);
    outTexCoord = tex;
}
"#;

pub const FRAGMENT_SHADER_SOURCE: &str = r#"
#version 330
out vec4 FragColor;
in vec2 outTexCoord;

uniform sampler2D atlas;

void main() {
    FragColor = texture(atlas, outTexCoord);
}
"#;

fn main() {
    let keyboard = Keyboard::new();
    if let Ok(mut window) = Window::init(SCR_WIDTH, SCR_HEIGHT, TITLE, keyboard) {
        if let Ok(shader_program_data) = get_shader_program(VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE) {
            let shader_program: Rc<ShaderProgram> = Rc::new(shader_program_data);
            if let Ok(blocks_loader) = get_blocks_loader(shader_program.clone()) {
                let world_chunk_generator: LayerChunkGenerator = LayerChunkGenerator::from_bottom_layers(&[BEDROCK_BLOCK_ID, DIRT_BLOCK_ID, DIRT_BLOCK_ID, DIRT_BLOCK_ID, GRASS_BLOCK_ID]);
                if let Ok(world) = World::new(&world_chunk_generator, &blocks_loader) {
                    if let Some(bedrock) = blocks_loader.blocks_ids.get(&GRASS_BLOCK_ID) {
                        if let Some(unknown) = blocks_loader.blocks_ids.get(&UNKNOWN_BLOCK_ID) {
                            world.set_block(&[0, 5, 10], unknown.lid);
                            world.set_block(&[0, 6, 10], unknown.lid);
                            world.set_block(&[0, 5, 9], unknown.lid);
                        } else {
                            return;
                        }
                    } else {
                        return;
                    }
                    let mut camera: Camera = Camera::new();
                    let fov: f32 = (60.0f32).to_radians();
                    let z_near: f32 = 0.01;
                    let z_far: f32 = 1000.0;
                    let asp_rat: f32 = (800.0 / 600.0);
                    let mut view_mat = Mat4f::new();
                    let mut proj_mat = Mat4f::new();
                    proj_mat.identity().perspective(fov, asp_rat, z_near, z_far);

                    unsafe {
                        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
                        gl::Enable(gl::BLEND);
                        gl::Enable(gl::DEPTH_TEST);
                    }
                    while !window.should_close() {
                        window.process_events();
                        window.swap_buffers();
                        camera.get_view_mat_to(&proj_mat, &mut view_mat);
                        unsafe {
                            if let Err(_) = shader_program.set_uniform_mat4f("viewMat", &view_mat) {
                                println!("Failed to load view matrix");
                            }
                        }
                        unsafe {
                            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
                        }
                        {
                            let mut move_pos_cam_vec: Vec3f = [0.0, 0.0, 0.0];
                            let mut move_rot_cam_vec: Vec3f = [0.0, 0.0, 0.0];

                            if window.keyboard.get_key_state(glfw::Key::Escape) {
                                window.close();
                            }


                            //TODO ROTSPEED MOVESPEED
                            if window.keyboard.get_key_state(glfw::Key::D) {
                                move_pos_cam_vec[0] += 0.05;
                            }
                            if window.keyboard.get_key_state(glfw::Key::A) {
                                move_pos_cam_vec[0] -= 0.05;
                            }

                            if window.keyboard.get_key_state(glfw::Key::Space) {
                                move_pos_cam_vec[1] += 0.05;
                            }
                            if window.keyboard.get_key_state(glfw::Key::LeftShift) || window.keyboard.get_key_state(glfw::Key::RightShift) {
                                move_pos_cam_vec[1] -= 0.05;
                            }

                            if window.keyboard.get_key_state(glfw::Key::S) {
                                move_pos_cam_vec[2] += 0.05;
                            }
                            if window.keyboard.get_key_state(glfw::Key::W) {
                                move_pos_cam_vec[2] -= 0.05;
                            }

                            if window.keyboard.get_key_state(glfw::Key::Down) {
                                //let cur_rot_x: f32 = camera.get_rotation_x();
                                move_rot_cam_vec[0] += 0.5;
                            }
                            if window.keyboard.get_key_state(glfw::Key::Up) {
                                //let cur_rot_x: f32 = camera.get_rotation_x();
                                move_rot_cam_vec[0] -= 0.5;
                            }

                            if window.keyboard.get_key_state(glfw::Key::Right) {
                                move_rot_cam_vec[1] += 0.5;
                            }
                            if window.keyboard.get_key_state(glfw::Key::Left) {
                                move_rot_cam_vec[1] -= 0.5;
                            }

                            if window.keyboard.get_key_state(glfw::Key::H) {
                                move_rot_cam_vec[2] += 0.5;
                            }
                            if window.keyboard.get_key_state(glfw::Key::Y) {
                                move_rot_cam_vec[2] -= 0.5;
                            }

                            camera.move_position(&move_pos_cam_vec);
                            camera.move_rotation(&move_rot_cam_vec);
                        }
                        if let Err(_) = world.render(&blocks_loader) {
                            println!("Failed to render world");
                        }
                        world.draw(&blocks_loader);
                    }
                    unsafe {
                        gl::Disable(gl::BLEND);
                        gl::Disable(gl::DEPTH_TEST);
                    }
                } else {
                    println!("Failed to create world");
                }
            } else {
                println!("Failed to load blocks data");
            }
        } else {
            println!("Failed to create shader program");
        }
    } else {
        println!("Failed to create window");
    }
}
