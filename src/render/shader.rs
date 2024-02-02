use std::ffi::CString;
use std::ptr;
use gl::types::{GLenum, GLint, GLuint};
use crate::render::types::ShaderError;

pub struct Shader {
    pub(crate) id: GLuint,
}

impl Shader {
    pub fn get_id(&self) -> GLuint {
        self.id
    }

    pub unsafe fn new(src: &str, shader_type: GLenum) -> Result<Self, ShaderError> {
        let src = CString::new(src)?;
        let shader = Self {
            id: gl::CreateShader(shader_type),
        };
        gl::ShaderSource(shader.id, 1, &src.as_ptr(), ptr::null());
        gl::CompileShader(shader.id);

        let mut success: GLint = 0;
        gl::GetShaderiv(shader.id, gl::COMPILE_STATUS, &mut success);

        if success == gl::TRUE as GLint {
            Ok(shader)
        } else {
            let mut error_log_size: GLint = 512;
            gl::GetShaderiv(shader.id, gl::INFO_LOG_LENGTH, &mut error_log_size);
            let mut error_log: Vec<u8> = Vec::with_capacity(error_log_size as usize);
            {
                gl::GetShaderInfoLog(
                    shader.id,
                    error_log_size,
                    &mut error_log_size,
                    error_log.as_mut_ptr() as *mut _,
                );
                error_log.set_len(error_log_size as usize); //Last null byte
            }
            let log = String::from_utf8(error_log)?;
            Err(ShaderError::CompilationError(log))
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.id);
        }
    }
}