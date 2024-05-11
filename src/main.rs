use std::path::Path;
use std::ptr;
use std::rc::Rc;
use glfw::ffi::KEY_ESCAPE;
use glfw::{MouseButtonLeft, MouseButtonRight};
use rand::distributions::uniform::SampleBorrow;
use rand::Rng;
use cubecode_a000::chunk::{Chunk, LayerChunkGenerator, SubChunk};
use cubecode_a000::input::keyboard::Keyboard;
use cubecode_a000::input::mouse::Mouse;
use cubecode_a000::render::blocks_loader::{AIR_BLOCK_ID, BEDROCK_BLOCK_ID, BlocksLoader, DIRT_BLOCK_ID, GRASS_BLOCK_ID, UNKNOWN_BLOCK_ID};
use cubecode_a000::render::buffer::Buffer;
use cubecode_a000::render::camera::Camera;
use cubecode_a000::render::faces_loader::FacesLoader;
use cubecode_a000::render::gui_renderer::GuiRenderer;
use cubecode_a000::render::light::light_map::{B_CHANNEL, G_CHANNEL, LightMap, R_CHANNEL, S_CHANNEL};
use cubecode_a000::render::light::light_solver::LightSolver;
use cubecode_a000::render::meshes_loader::MeshesLoader;
use cubecode_a000::render::shader::Shader;
use cubecode_a000::render::shader_program::ShaderProgram;
use cubecode_a000::render::types::{Mat4f, Vec3f, Vec3ub, LightedTexVertex, Vec3s, Vec3b, add_vec3f, sub_vec3f, norm_vec3f, Vec2d};
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
in vec4 light;

out vec4 col;
out vec2 outTexCoord;

uniform mat4 viewMat;
uniform mat4 modelMat;

void main() {
    gl_Position = viewMat * modelMat * vec4(pos, 1.0);
    col = vec4(light.r, light.g, light.b, 1.0f);
    col.rgb += light.a;
    outTexCoord = tex;
}
"#;

pub const FRAGMENT_SHADER_SOURCE: &str = r#"
#version 330
out vec4 FragColor;
in vec2 outTexCoord;
in vec4 col;

uniform sampler2D atlas;

void main() {
    FragColor = col * texture(atlas, outTexCoord);
}
"#;

const NEIGHBORHOOD: [Vec3b; 6] = [[0, 0, -1], [0, 0, 1], [0, -1, 0], [0, 1, 0], [-1, 0, 0], [1, 0, 0]];

fn main() {
    let keyboard: Keyboard = Keyboard::new();
    let mouse: Mouse = Mouse::new();
    if let Ok(mut window) = Window::init(SCR_WIDTH, SCR_HEIGHT, TITLE, keyboard, mouse) {
        if let Ok(shader_program_data) = get_shader_program(VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE) {
            let shader_program: Rc<ShaderProgram> = Rc::new(shader_program_data);
            if let Ok(blocks_loader) = get_blocks_loader(shader_program.clone()) {
                let world_chunk_generator: LayerChunkGenerator = LayerChunkGenerator::from_bottom_layers(&[BEDROCK_BLOCK_ID, DIRT_BLOCK_ID, DIRT_BLOCK_ID, DIRT_BLOCK_ID, GRASS_BLOCK_ID]);
                if let Ok(world) = World::new(&world_chunk_generator, &blocks_loader) {
                    if let Ok(gui_renderer) = GuiRenderer::init_gui() {
                        if let Some(bedrock) = blocks_loader.blocks_ids.get(&GRASS_BLOCK_ID) {
                            if let Some(unknown) = blocks_loader.blocks_ids.get(&UNKNOWN_BLOCK_ID) {
                                world.load(&blocks_loader, String::from("world/world.data")).unwrap();
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
                        let z_far: f32 = 1024.0;
                        let mut asp_rat: f32 = (800.0 / 600.0);
                        let mut view_mat = Mat4f::new();
                        let mut proj_mat = Mat4f::new();
                        let mut cur_lid: u16 = UNKNOWN_BLOCK_ID;
                        proj_mat.identity().perspective(fov, asp_rat, z_near, z_far);
                        gui_renderer.set_asp_rat(asp_rat);

                        //rewrite to other init_light_solvers function
                        let solver_r = LightSolver::new(R_CHANNEL);
                        let solver_g = LightSolver::new(G_CHANNEL);
                        let solver_b = LightSolver::new(B_CHANNEL);
                        let solver_s = LightSolver::new(S_CHANNEL);

                        for x_pos in 0x00_u8..=0xFF_u8 {
                            for y_pos in 0x00_u8..=0xFF_u8 {
                                for z_pos in 0x00_u8..=0xFF_u8 {
                                    let pos: Vec3ub = [x_pos, y_pos, z_pos];
                                    let block_lid = world.get_block(&pos);
                                    let mut light_r: u8 = 0;
                                    let mut light_g: u8 = 0;
                                    let mut light_b: u8 = 0;
                                    if let Some(block) = blocks_loader.loaded_blocks.get(block_lid as usize) {
                                        light_r = block.light_r;
                                        light_g = block.light_g;
                                        light_b = block.light_b;
                                    } else {
                                        world.set_block(&pos, UNKNOWN_BLOCK_ID);
                                        if let Some(block) = blocks_loader.loaded_blocks.get(UNKNOWN_BLOCK_ID as usize) {
                                            light_r = block.light_r;
                                            light_g = block.light_g;
                                            light_b = block.light_b;
                                        } else {
                                            println!("FATAL ERROR 0");
                                            return;
                                        }
                                    }
                                    if light_r != 0 {solver_r.add(&world, &pos, light_r);}
                                    if light_g != 0 {solver_g.add(&world, &pos, light_g);}
                                    if light_b != 0 {solver_b.add(&world, &pos, light_b);}
                                }
                            }
                        }

                        for x_pos in 0x00_u8..=0xFF_u8 {
                            for z_pos in 0x00_u8..=0xFF_u8 {
                                for y_pos in (0x00_u8..=0xFF_u8).rev() {
                                    let pos: Vec3ub = [x_pos, y_pos, z_pos];
                                    let block_lid: u16 = world.get_block(&pos);
                                    if blocks_loader.get_block(block_lid).mesh.is_cube() {
                                        break;
                                    }
                                    world.set_light_level(&pos, S_CHANNEL, 0x0F);
                                }
                            }
                        }

                        for x_pos in 0x00_u8..=0xFF_u8 {
                            for z_pos in 0x00_u8..=0xFF_u8 {
                                for y_pos in (0x00_u8..=0xFF_u8).rev() {
                                    let pos: Vec3ub = [x_pos, y_pos, z_pos];
                                    let block_lid: u16 = world.get_block(&pos);
                                    if blocks_loader.get_block(block_lid).mesh.is_cube() {
                                        break;
                                    }

                                    let mut flag: bool = false;
                                    for neigh in NEIGHBORHOOD {
                                        if let Some(neigh_pos) = LightSolver::get_neighbor_pos(&pos, &neigh) {
                                            if world.get_light_level(&neigh_pos, S_CHANNEL) == 0 {
                                                flag = true;
                                                break;
                                            }
                                        }
                                    }
                                    if flag {
                                        solver_s.add_last(&world, &pos);
                                    }
                                }
                            }
                        }

                        solver_r.solve(&world, &blocks_loader);
                        solver_g.solve(&world, &blocks_loader);
                        solver_b.solve(&world, &blocks_loader);
                        solver_s.solve(&world, &blocks_loader);
                        //

                        unsafe {
                            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
                            gl::Enable(gl::BLEND);
                            gl::Enable(gl::DEPTH_TEST);
                        }
                        let mut bflag: bool = true;
                        let mut dflag: bool = true;
                        let mut tab_flag: bool = true;
                        while !window.should_close() {
                            window.process_events();
                            window.swap_buffers();
                            if asp_rat != window.asp_rat {
                                asp_rat = window.asp_rat;
                                proj_mat.identity().perspective(fov, asp_rat, z_near, z_far);
                                gui_renderer.set_asp_rat(asp_rat);
                            }
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
                                let mut move_pos_cam_dir: Vec3f = [0.0, 0.0, 0.0];
                                let mut move_rot_cam_vec: Vec3f = [0.0, 0.0, 0.0];

                                if window.keyboard.get_key_state(glfw::Key::Escape) {
                                    window.close();
                                }


                                //TODO ROTSPEED MOVESPEED
                                if window.keyboard.get_key_state(glfw::Key::D) {
                                    add_vec3f(&mut move_pos_cam_dir, camera.get_rdir());
                                }
                                if window.keyboard.get_key_state(glfw::Key::A) {
                                    sub_vec3f(&mut move_pos_cam_dir, camera.get_rdir());
                                }

                                if window.keyboard.get_key_state(glfw::Key::Space) {
                                    add_vec3f(&mut move_pos_cam_dir, camera.get_udir());
                                }
                                if window.keyboard.get_key_state(glfw::Key::LeftShift) || window.keyboard.get_key_state(glfw::Key::RightShift) {
                                    sub_vec3f(&mut move_pos_cam_dir, camera.get_udir());
                                }

                                if window.keyboard.get_key_state(glfw::Key::S) {
                                    sub_vec3f(&mut move_pos_cam_dir, camera.get_fdir());
                                }
                                if window.keyboard.get_key_state(glfw::Key::W) {
                                    add_vec3f(&mut move_pos_cam_dir, camera.get_fdir());
                                }
                                if window.mouse.borrow().get_cursor_state() {
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
                                } else {
                                    let pos: Vec2d = window.mouse.borrow().get_cursor_delta_pos().clone();
                                    let min_side = window.get_width().min(window.get_height());
                                    //86 - sensitivity
                                    move_rot_cam_vec[0] = ((pos[1] as f32) / (min_side as f32)) * 86.0f32;
                                    move_rot_cam_vec[1] = ((pos[0] as f32) / (min_side as f32)) * 86.0f32;
                                }

                                if window.keyboard.get_key_state(glfw::Key::Num1) {
                                    cur_lid = UNKNOWN_BLOCK_ID;
                                }

                                if window.keyboard.get_key_state(glfw::Key::Num2) {
                                    cur_lid = DIRT_BLOCK_ID;
                                }

                                if window.keyboard.get_key_state(glfw::Key::Num3) {
                                    cur_lid = GRASS_BLOCK_ID;
                                }

                                if window.keyboard.get_key_state(glfw::Key::Num4) {
                                    cur_lid = BEDROCK_BLOCK_ID;
                                }

                                if window.mouse.borrow().get_button_state(MouseButtonRight) {
                                    if dflag || window.keyboard.get_key_state(glfw::Key::R) {
                                        dflag = false;
                                        let mut end: Vec3f = [0.0, 0.0, 0.0];
                                        let mut norm: Vec3f = [0.0, 0.0, 0.0];
                                        let mut iend: Vec3ub = [0, 0, 0];
                                        world.ray_get(&camera.get_position(), &camera.get_dir(), 16.0, &mut end, &mut norm, &mut iend);
                                        world.set_block(&iend, AIR_BLOCK_ID);
                                        solver_r.remove(&world, &iend);
                                        solver_g.remove(&world, &iend);
                                        solver_b.remove(&world, &iend);
                                        solver_r.solve(&world, &blocks_loader);
                                        solver_g.solve(&world, &blocks_loader);
                                        solver_b.solve(&world, &blocks_loader);
                                        //TODO rename [0, 1, 0] to neighbor top
                                        let mut flag: bool = false;
                                        if let Some(neigh_pos) = LightSolver::get_neighbor_pos(&iend, &[0, 1, 0]) {
                                            if world.get_light_level(&neigh_pos, S_CHANNEL) == 0x0F {
                                                flag = true;
                                            }
                                        } else {
                                            flag = true;
                                        }
                                        if flag {
                                            for y_pos in (0x00..=iend[1]).rev() {
                                                let spos: Vec3ub = [iend[0], y_pos, iend[2]];
                                                let block_lid: u16 = world.get_block(&spos);
                                                if blocks_loader.get_block(block_lid).mesh.is_cube() {
                                                    break;
                                                }
                                                solver_s.add(&world, &spos, 0x0F);
                                            }
                                        }
                                        for neigh in NEIGHBORHOOD {
                                            if let Some(neigh_pos) = LightSolver::get_neighbor_pos(&iend, &neigh) {
                                                solver_r.add_last(&world, &neigh_pos);
                                                solver_g.add_last(&world, &neigh_pos);
                                                solver_b.add_last(&world, &neigh_pos);
                                                solver_s.add_last(&world, &neigh_pos);
                                            }
                                        }
                                        solver_r.solve(&world, &blocks_loader);
                                        solver_g.solve(&world, &blocks_loader);
                                        solver_b.solve(&world, &blocks_loader);
                                        solver_s.solve(&world, &blocks_loader);
                                    }
                                } else {
                                    dflag = true;
                                }

                                if window.mouse.borrow().get_button_state(MouseButtonLeft) {
                                    if bflag || window.keyboard.get_key_state(glfw::Key::R) {
                                        bflag = false;
                                        let mut end: Vec3f = [0.0, 0.0, 0.0];
                                        let mut norm: Vec3f = [0.0, 0.0, 0.0];
                                        let mut iend: Vec3ub = [0, 0, 0];
                                        if let Some(block) = world.ray_get(&camera.get_position(), &camera.get_dir(), 16.0, &mut end, &mut norm, &mut iend) {
                                            if block != AIR_BLOCK_ID {
                                                let res: Vec3s = [(iend[0] as i16 + norm[0] as i16), (iend[1] as i16 + norm[1] as i16), (iend[2] as i16 + norm[2] as i16)];
                                                if res[0] >= 0x00 && res[0] <= 0xFF &&
                                                    res[1] >= 0x00 && res[1] <= 0xFF &&
                                                    res[2] >= 0x00 && res[2] <= 0xFF {
                                                    let pos: Vec3ub = [res[0] as u8, res[1] as u8, res[2] as u8];
                                                    if world.get_block(&pos) == AIR_BLOCK_ID {
                                                        let block = blocks_loader.get_block(cur_lid);
                                                        world.set_block(&pos, block.lid);
                                                        solver_r.remove(&world, &pos);
                                                        solver_g.remove(&world, &pos);
                                                        solver_b.remove(&world, &pos);
                                                        solver_s.remove(&world, &pos);
                                                        //TODO rename [0, -1, 0] to neighbor bottom

                                                        //TODO REWRITE IT

                                                        //PLACE WITH THE MOST WTF ERROR (IEND CONFUSED WITH POS)
                                                        if let Some(neigh_pos) = LightSolver::get_neighbor_pos(&pos, &[0, -1, 0]) {
                                                            for y_pos in (0x00..=neigh_pos[1]).rev() {
                                                                let spos: Vec3ub = [pos[0], y_pos, pos[2]];
                                                                solver_s.remove(&world, &spos);
                                                                if let Some(bottom_spos) = LightSolver::get_neighbor_pos(&spos, &[0, -1, 0]) {
                                                                    let block_lid: u16 = world.get_block(&bottom_spos);
                                                                    if blocks_loader.get_block(block_lid).mesh.is_cube() {
                                                                        break;
                                                                    }
                                                                } else if y_pos == 0 {
                                                                    break;
                                                                }
                                                            }
                                                            solver_r.solve(&world, &blocks_loader);
                                                            solver_g.solve(&world, &blocks_loader);
                                                            solver_b.solve(&world, &blocks_loader);
                                                            solver_s.solve(&world, &blocks_loader);
                                                            if block.light_r != 0 {
                                                                solver_r.add(&world, &pos, block.light_r);
                                                                solver_r.solve(&world, &blocks_loader);
                                                            }
                                                            if block.light_g != 0 {
                                                                solver_g.add(&world, &pos, block.light_g);
                                                                solver_r.solve(&world, &blocks_loader);
                                                            }
                                                            if block.light_b != 0 {
                                                                solver_b.add(&world, &pos, block.light_b);
                                                                solver_r.solve(&world, &blocks_loader);
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    bflag = true;
                                }


                                if window.keyboard.get_key_state(glfw::Key::F) {
                                    if let Err(error) = world.store(&blocks_loader, String::from("world/world.data")) {
                                        println!("Failed to save the world");
                                    } else {
                                        println!("World has been saved successfully");
                                    }
                                }

                                if window.keyboard.get_key_state(glfw::Key::G) {
                                    if let Err(error) = world.load(&blocks_loader, String::from("world/world.data")) {
                                        println!("Failed to load the world");
                                    } else {
                                        println!("World has been loaded successfully");
                                    }
                                }

                                if window.keyboard.get_key_state(glfw::Key::Q) {
                                    world.set_block(&[8, 8, 8], rand::thread_rng().gen_range(0..4) as u16);
                                }

                                if window.keyboard.get_key_state(glfw::Key::M) {
                                    println!("{:?}, {:?}", camera.get_position(), camera.get_rotation());
                                }

                                if window.keyboard.get_key_state(glfw::Key::B) {
                                    let mut end: Vec3f = [0.0, 0.0, 0.0];
                                    let mut norm: Vec3f = [0.0, 0.0, 0.0];
                                    let mut iend: Vec3ub = [0, 0, 0];
                                    if let Some(block) = world.ray_get(&camera.get_position(), &camera.get_dir(), 16.0, &mut end, &mut norm, &mut iend) {
                                        if block != AIR_BLOCK_ID {
                                            println!("lid: {}, r: {}, g: {}, b: {}, s: {}", world.get_block(&iend), world.get_light_level(&iend, 0), world.get_light_level(&iend, 1), world.get_light_level(&iend, 2), world.get_light_level(&iend, 3));
                                        }
                                    }
                                }

                                if window.keyboard.get_key_state(glfw::Key::L) {
                                    let mut end: Vec3f = [0.0, 0.0, 0.0];
                                    let mut norm: Vec3f = [0.0, 0.0, 0.0];
                                    let mut iend: Vec3ub = [0, 0, 0];
                                    if let Some(block) = world.ray_get(&camera.get_position(), &camera.get_dir(), 16.0, &mut end, &mut norm, &mut iend) {
                                        solver_r.add(&world, &iend, 0x0F);
                                    }
                                }

                                if window.keyboard.get_key_state(glfw::Key::Tab) {
                                    if tab_flag {
                                        tab_flag = false;
                                        window.mouse.borrow_mut().toggle_cursor(&window);
                                    }
                                } else {
                                    tab_flag = true;
                                }
                                norm_vec3f(&mut move_pos_cam_dir);
                                camera.move_position(&move_pos_cam_dir, 0.05);
                                camera.move_rotation(&move_rot_cam_vec);
                            }
                            solver_r.solve(&world, &blocks_loader);
                            solver_g.solve(&world, &blocks_loader);
                            solver_b.solve(&world, &blocks_loader);
                            solver_s.solve(&world, &blocks_loader);
                            if let Err(_) = world.render(&blocks_loader) {
                                println!("Failed to render world");
                            }
                            world.draw(&blocks_loader);
                            if let Err(_) = gui_renderer.render() {
                                println!("Failed to render GUI");
                            }
                            gui_renderer.draw();
                        }
                        unsafe {
                            gl::Disable(gl::BLEND);
                            gl::Disable(gl::DEPTH_TEST);
                        }
                    } else {
                        println!("Failed to initialize GUI");
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
