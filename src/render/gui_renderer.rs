use std::rc::Rc;
use crate::render::shader_program::ShaderProgram;

const XPOINTER_VERTICES: [f32; 8] = [
    0.00f32, 0.01f32,
    0.00f32, -0.01f32,

    0.01f32, 0.01f32,
    0.01f32, -0.01f32,
];

pub const VERTEX_GUI_SHADER_SOURCE: &str = r#"
#version 330

layer (location = 0) in vec2 pos;

void main() {
    gl_Position = vec4(pos, 0.0, 1.0);
}
"#;

pub const FRAGMENT_GUI_SHADER_SOURCE: &str = r#"
#version 330

out vec4 f_color;

void main() {
    f_color = vec4(f_color, 1.0);
}
"#;

pub struct GuiRenderer {
    pub shader_program: Rc<ShaderProgram>,
}

impl GuiRenderer {
    /*fn init_gui() -> Result<ShaderProgram, Box<dyn std::error::Error>> {
         match get_shader_program(VERTEX_SHADER_GUI_SOURCE, FRAGMENT_SHADER_GUI_SOURCE) {
             Ok(shader_program_data) => {

             }
             Err
            let shader_program: Rc<ShaderProgram> = Rc::new(shader_program_data);
            return Ok(Self {
                shader_program,
            });
        } else {
            return Err()
        }
    }*/
}