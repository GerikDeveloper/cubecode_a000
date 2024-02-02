use std::ffi::{CString, NulError};
use std::ptr;
use gl::types::{GLfloat, GLint, GLuint};
use crate::render::shader::Shader;
use crate::render::types::{Mat4f, ShaderError};

pub struct ShaderProgram {
    pub(crate) id: GLuint,
}

impl ShaderProgram {
    pub fn get_id(&self) -> GLuint {
        self.id
    }

    pub unsafe fn new(shaders: &[Shader]) -> Result<Self, ShaderError> {
        let program = Self {
            id: gl::CreateProgram(),
        };

        for shader in shaders {
            gl::AttachShader(program.id, shader.id);
        }

        gl::LinkProgram(program.id);

        let mut success: GLint = 0;
        gl::GetProgramiv(program.id, gl::LINK_STATUS, &mut success);

        if success == gl::TRUE as GLint {
            Ok(program)
        } else{
            let mut error_log_size: GLint = 512;
            gl::GetProgramiv(program.id, gl::INFO_LOG_LENGTH, &mut error_log_size);
            let mut error_log: Vec<u8> = Vec::with_capacity(error_log_size as usize);
            gl::GetProgramInfoLog(
                program.id,
                error_log_size,
                &mut error_log_size,
                error_log.as_mut_ptr() as *mut _,
            );
            error_log.set_len(error_log_size as usize);
            let log = String::from_utf8(error_log)?;
            Err(ShaderError::LinkingError(log))
        }
    }

    pub unsafe fn apply(&self) {
        gl::UseProgram(self.id);
    }

    pub unsafe fn get_attrib_location(&self, attrib: &str) -> Result<GLuint, NulError> {
        let attrib = CString::new(attrib)?;
        Ok(gl::GetAttribLocation(self.id, attrib.as_ptr()) as GLuint)
    }

    pub unsafe fn set_uniform_i32(&self, name: &str, value: i32) -> Result<(), ShaderError> {
        self.apply();
        let uniform = CString::new(name)?;
        gl::Uniform1i(gl::GetUniformLocation(self.id, uniform.as_ptr()), value);
        Ok(())
    }

    pub unsafe fn set_uniform_mat4f(&self, name: &str, value: &Mat4f) -> Result<(), ShaderError> {
        self.apply();
        let uniform = CString::new(name)?;
        gl::UniformMatrix4fv(gl::GetUniformLocation(self.id, uniform.as_ptr()), 1, gl::FALSE, ptr::addr_of!(value.matrix) as *const GLfloat);
        Ok(())
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

#[macro_export]
macro_rules! set_attribute {
    //vbo was ident
    ($vbo:expr, $pos:tt, $t:ident :: $field:tt) => {
        {
            let dummy = core::mem::MaybeUninit::<$t>::uninit();
            let dummy_ptr = dummy.as_ptr();
            let member_ptr = core::ptr::addr_of!((*dummy_ptr).$field);
            const fn size_of_raw<T>(_: *const T) -> usize {
                core::mem::size_of::<T>()
            }
            let member_offset = (member_ptr as i32) - (dummy_ptr as i32);
            $vbo.set_attribute::<$t>(
                $pos,
                (size_of_raw(member_ptr) / core::mem::size_of::<f32>()) as i32,
                member_offset,
            )
        }
    };
}