use std::cell::{Cell, RefCell};
use std::ptr;
use std::rc::Rc;
use crate::render::block_renderer;
use crate::render::buffer::Buffer;
use crate::render::shader::Shader;
use crate::render::shader_program::ShaderProgram;
use crate::render::types::{ColorizedVertex, ShaderError, LightedTexVertex, Vec2f};
use crate::render::vertex_array::VertexArray;
use crate::set_attribute;

//temp
//TODO: rewrite xpointer and others to json and load it TODO another .rs for GUI element/widget and draw it in pos by translation
const XPOINTER_VERTICES: [ColorizedVertex; 8] = [
    ColorizedVertex([0.001f32, 0.01f32], [1.0f32, 1.0f32, 1.0f32]),
    ColorizedVertex([-0.001f32, 0.01f32], [1.0f32, 1.0f32, 1.0f32]),


    ColorizedVertex([0.001f32, -0.01f32], [1.0f32, 1.0f32, 1.0f32]),
    ColorizedVertex([-0.001f32, -0.01f32], [1.0f32, 1.0f32, 1.0f32]),

    ColorizedVertex([0.01f32, 0.001f32], [1.0f32, 1.0f32, 1.0f32]),
    ColorizedVertex([0.01f32, -0.001f32], [1.0f32, 1.0f32, 1.0f32]),

    ColorizedVertex([-0.01f32, 0.001f32], [1.0f32, 1.0f32, 1.0f32]),
    ColorizedVertex([-0.01f32, -0.001f32], [1.0f32, 1.0f32, 1.0f32]),
];

const XPOINTER_INDICES: [i32; 12] = [
    0, 1, 3,    2, 3, 0,
    4, 5, 7,    6, 7, 4,
];

struct ColorizedGuiElement {
    pos: Vec2f,
    vertices: Vec<ColorizedVertex>,
    indices: Vec<i32>,
    renderer: Rc<GuiRenderer>,
}

pub const COLORIZED_VERTEX_GUI_SHADER_SOURCE: &str = r#"
#version 330

in vec2 pos;
in vec3 col;

out vec3 out_color;

void main() {
    gl_Position = vec4(pos, 0.0, 1.0);
    out_color = col;
}
"#;

pub const COLORIZED_FRAGMENT_GUI_SHADER_SOURCE: &str = r#"
#version 330

in vec3 out_color;
out vec4 f_color;

void main() {
    f_color = vec4(out_color, 1.0);
}
"#;

pub struct GuiRenderer {
    pub shader_program: Rc<ShaderProgram>,
    pub gui_elements: RefCell<Vec<ColorizedGuiElement>>,
    pub is_changed: Cell<bool>,
    pub vert_buf: RefCell<Buffer>,
    pub ind_buf: RefCell<Buffer>,
    pub ind_cnt: Cell<i32>,
    pub vert_array: RefCell<VertexArray>,
    asp_rat: Cell<f32>,
}

impl GuiRenderer {
    fn get_shader_program(vertex_src: &str , fragment_src: &str) -> Result<ShaderProgram, Box<dyn std::error::Error>> {
        unsafe {
            let vert_shader = Shader::new(vertex_src, gl::VERTEX_SHADER)?;
            let frag_shader = Shader::new(fragment_src, gl::FRAGMENT_SHADER)?;
            let shader_program = ShaderProgram::new(&[vert_shader, frag_shader])?;
            return Ok(shader_program);
        }
    }
    pub fn init_gui() -> Result<Rc<Self>, Box<dyn std::error::Error>> {
        return match GuiRenderer::get_shader_program(COLORIZED_VERTEX_GUI_SHADER_SOURCE, COLORIZED_FRAGMENT_GUI_SHADER_SOURCE) {
            Ok(shader_program_data) => {
                let shader_program: Rc<ShaderProgram> = Rc::from(shader_program_data);
                unsafe {
                    let res: Rc<Self> = Rc::from(Self {
                        shader_program,
                        gui_elements: RefCell::from(Vec::new()),
                        is_changed: Cell::new(true),
                        vert_buf: RefCell::new(Buffer::new(gl::ARRAY_BUFFER)),
                        ind_buf: RefCell::new(Buffer::new(gl::ELEMENT_ARRAY_BUFFER)),
                        ind_cnt: Cell::new(0),
                        vert_array: RefCell::new(VertexArray::new()),
                        asp_rat: Cell::new(1.0f32),
                    });
                    let xpointer: ColorizedGuiElement = ColorizedGuiElement {
                        pos: [0.0f32, 0.0f32],
                        vertices: Vec::from(XPOINTER_VERTICES),
                        indices: Vec::from(XPOINTER_INDICES),
                        renderer: res.clone(),
                    };
                    GuiRenderer::add_colorized_gui_element(res.clone(), xpointer);
                    Ok(res)
                }
            }
            Err(shader_program_error) => {
                Err(shader_program_error)
            }
        }
    }

    pub fn add_colorized_gui_element(renderer: Rc<Self>, mut gui_element: ColorizedGuiElement) {
        gui_element.renderer = renderer.clone();
        renderer.gui_elements.borrow_mut().push(gui_element);
    }

    pub fn set_asp_rat(&self, asp_rat: f32) {
        self.asp_rat.replace(asp_rat);
        self.is_changed.replace(true);
    }

    pub fn render(&self) -> Result<(), Vec<Box<dyn std::error::Error>>> {
        let mut errors: Vec<Box<dyn std::error::Error>> = Vec::new();
        if self.is_changed.get() {
            let mut vertices: Vec<ColorizedVertex> = Vec::new();
            let mut indices: Vec<i32> = Vec::new();
            for gui_element in self.gui_elements.borrow().iter() {
                for ver in &gui_element.vertices {
                    let asp_pos: Vec2f = [(ver.0[0] + gui_element.pos[0]), (ver.0[1] + gui_element.pos[1]) * self.asp_rat.get()];
                    vertices.push(ColorizedVertex(asp_pos, ver.1));
                }
                for ind in &gui_element.indices {indices.push(*ind);}
            }
            unsafe {
                self.vert_array.replace(VertexArray::new());
                self.vert_array.borrow().bind();

                self.vert_buf.replace(Buffer::new(gl::ARRAY_BUFFER));
                self.vert_buf.borrow().set_data(vertices.as_slice(), gl::STATIC_DRAW);

                match self.shader_program.get_attrib_location("pos") {
                    Ok(pos_attrib) => {

                        set_attribute!(self.vert_array.borrow(), pos_attrib, ColorizedVertex::0);

                        match self.shader_program.get_attrib_location("col") {
                            Ok(col_attrib) => {

                                set_attribute!(self.vert_array.borrow(), col_attrib, ColorizedVertex::1);

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

            self.ind_cnt.set(indices.len() as i32);
            self.is_changed.set(false);
        }

        return if errors.len() == 0 {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn draw(&self) {
        if self.ind_cnt.get() != 0 {
            unsafe {
                self.shader_program.apply();
                self.vert_array.borrow().bind();
                gl::DrawElements(gl::TRIANGLES, self.ind_cnt.get(), gl::UNSIGNED_INT, ptr::null());
            }
        }
    }
}