use std::cell::{Cell, RefCell, RefMut};
use std::ptr;
use std::rc::Rc;
use crate::render::buffer::Buffer;
use crate::render::shader::Shader;
use crate::render::shader_program::ShaderProgram;
use crate::render::types::{RGBALine, Mat4f, RGBAVertex3f, Vec3f, Vec3ub};
use crate::render::vertex_array::VertexArray;
use crate::set_attribute;


//TODO 3dGUI RENDERER
pub struct LinesRenderer {
    pub shader_program: Rc<ShaderProgram>,
    pub vertices: RefCell<Vec<RGBAVertex3f>>,
    pub indices: RefCell<Vec<i32>>,
    pub is_changed: Cell<bool>,
    pub vert_buf: RefCell<Buffer>,
    pub ind_buf: RefCell<Buffer>,
    pub ind_cnt: Cell<i32>,
    pub vert_array: RefCell<VertexArray>,
}

pub const COLORIZED_VERTEX_LINES_SHADER_SOURCE: &str = r#"
#version 330

in vec3 pos;
in vec4 col;

out vec4 out_col;

uniform mat4 viewMat;

void main() {
    gl_Position = viewMat * vec4(pos, 1.0);
    out_col = col;
}
"#;

pub const COLORIZED_FRAGMENT_LINES_SHADER_SOURCE: &str = r#"
#version 330

out vec4 FragColor;

in vec4 out_col;

void main() {
    FragColor = out_col;
}
"#;

//TODO MB REPLACE WITH BIT SHIFTING
const CUBE_VERTICES_SHIFT: [[f32; 3]; 8] = [
    [0.0, 0.0, 0.0], //0
    [0.0, 0.0, 1.0], //1
    [0.0, 1.0, 0.0], //2
    [0.0, 1.0, 1.0], //3
    [1.0, 0.0, 0.0], //4
    [1.0, 0.0, 1.0], //5
    [1.0, 1.0, 0.0], //6
    [1.0, 1.0, 1.0], //7
];

const CUBE_LINES_INDICES: [i32; 24] = [
    0, 1,
    0, 2,
    0, 4,
    5, 1,
    5, 4,
    5, 7,
    6, 2,
    6, 7,
    6, 4,
    3, 7,
    3, 2,
    3, 1,
];

impl LinesRenderer {

    fn get_shader_program(vertex_src: &str , fragment_src: &str) -> Result<ShaderProgram, Box<dyn std::error::Error>> {
        unsafe {
            let vert_shader: Shader = Shader::new(vertex_src, gl::VERTEX_SHADER)?;
            let frag_shader: Shader = Shader::new(fragment_src, gl::FRAGMENT_SHADER)?;
            let shader_program: ShaderProgram = ShaderProgram::new(&[vert_shader, frag_shader])?;
            return Ok(shader_program);
        }
    }

    pub fn init_lines_renderer() -> Result<Self, Box<dyn std::error::Error>> {
        let shader_program: ShaderProgram = Self::get_shader_program(COLORIZED_VERTEX_LINES_SHADER_SOURCE,
                                                                     COLORIZED_FRAGMENT_LINES_SHADER_SOURCE)?;
        unsafe {
            return Ok(Self {
                shader_program: Rc::from(shader_program),
                vertices: RefCell::from(Vec::new()),
                indices: RefCell::from(Vec::new()),
                is_changed: Cell::new(false),
                vert_buf: RefCell::new(Buffer::new(gl::ARRAY_BUFFER)),
                ind_buf: RefCell::new(Buffer::new(gl::ELEMENT_ARRAY_BUFFER)),
                ind_cnt: Cell::new(0),
                vert_array: RefCell::new(VertexArray::new()),
            });
        }
    }

    pub fn render(&self) -> Result<(), Vec<Box<dyn std::error::Error>>> {
        let mut errors: Vec<Box<dyn std::error::Error>> = Vec::new();
        if self.is_changed.get() {
            unsafe {
                self.vert_array.replace(VertexArray::new());
                self.vert_array.borrow().bind();

                self.vert_buf.replace(Buffer::new(gl::ARRAY_BUFFER));
                self.vert_buf.borrow().set_data(self.vertices.borrow().as_slice(), gl::STATIC_DRAW);

                match self.shader_program.get_attrib_location("pos") {
                    Ok(pos_attrib) => {

                        set_attribute!(self.vert_array.borrow(), pos_attrib, RGBAVertex3f::0);

                        match self.shader_program.get_attrib_location("col") {
                            Ok(tex_attrib) => {

                                set_attribute!(self.vert_array.borrow(), tex_attrib, RGBAVertex3f::1);

                                self.ind_buf.replace(Buffer::new(gl::ELEMENT_ARRAY_BUFFER));
                                self.ind_buf.borrow().set_data(self.indices.borrow().as_slice(), gl::STATIC_DRAW);
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

            self.ind_cnt.set(self.indices.borrow().len() as i32);
            self.is_changed.set(false);
        }

        return if errors.len() == 0 {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn set_lines_width(&self, width: f32) {
        unsafe{gl::LineWidth(width);}
    }

    pub fn set_view_mat(&self, view_mat: &Mat4f) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            return if let Err(error) = self.shader_program.set_uniform_mat4f("viewMat", view_mat) {
                Err(Box::from(error))
            } else {
                Ok(())
            }
        }
    }

    pub fn add_line(&self, line: &RGBALine) {
        let mut vertices: RefMut<Vec<RGBAVertex3f>> = self.vertices.borrow_mut();
        let mut indices: RefMut<Vec<i32>> = self.indices.borrow_mut();
        indices.push((vertices.len() as i32));
        vertices.push(line[0].clone());
        indices.push((vertices.len() as i32));
        vertices.push(line[1].clone());
        self.is_changed.set(true);
    }

    fn add_vertex(&self, vertex: &RGBAVertex3f) {
        let mut vertices: RefMut<Vec<RGBAVertex3f>> = self.vertices.borrow_mut();
        self.indices.borrow_mut().push((vertices.len() as i32));
        vertices.push(vertex.clone());
    }

    //TODO is_changed = true after setting struct everywhere

    pub fn add_box(&self, pos: &Vec3ub) {
        let mut vertices: RefMut<Vec<RGBAVertex3f>> = self.vertices.borrow_mut();
        let first_ind: i32 = (vertices.len() as i32);
        let mut indices: RefMut<Vec<i32>> = self.indices.borrow_mut();
        for vertex in CUBE_VERTICES_SHIFT {
            let res_pos: Vec3f = [((pos[0] as f32) + vertex[0]), ((pos[1] as f32) + vertex[1]), ((pos[2] as f32) + vertex[2])];
            vertices.push(RGBAVertex3f(res_pos, [0.0, 0.0, 0.0, 1.0]));
        }
        for ind in CUBE_LINES_INDICES {
            indices.push(first_ind + ind);
        }
        self.is_changed.set(true);
    }

    pub fn clear(&self) {
        self.vertices.borrow_mut().clear();
        self.indices.borrow_mut().clear();
        self.is_changed.set(true);
    }

    pub fn draw(&self) {
        if self.ind_cnt.get() != 0 {
            unsafe {
                self.shader_program.apply();
                self.vert_array.borrow().bind();
                gl::DrawElements(gl::LINES, self.ind_cnt.get(), gl::UNSIGNED_INT, ptr::null());
            }
        }
    }
}