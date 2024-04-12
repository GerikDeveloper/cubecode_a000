use std::sync::mpsc::Receiver;
use glfw::{Action, Context, Glfw, Key, WindowEvent};
use thiserror::Error;
use crate::input::keyboard::Keyboard;

#[derive(Error, Debug)]
pub enum WinitError {
    #[error("Window creation failed")]
    CreationError(),
}

pub struct Window {
    pub glfw: Glfw,
    pub window: glfw::Window,
    pub events: Receiver<(f64, WindowEvent)>,
    pub events_processor: fn(&mut Window),
    pub keyboard: Keyboard,
    pub asp_rat: f32,
}

impl Window {
    pub fn init(width: u32, height: u32, title: &str, keyboard: Keyboard) -> Result<Self, Box<dyn std::error::Error>> {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS)?;
        glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
        //#[cfg(target_os = "macos")]
        glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
        if let Some((mut window, mut events)) = glfw.create_window(width, height, title, glfw::WindowMode::Windowed) {
            window.make_current();
            window.set_key_polling(true);
            window.set_framebuffer_size_polling(true);
            gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
            Ok(Self {
                glfw,
                window,
                events,
                events_processor: |wnd| Self::default_events_processor(wnd),
                keyboard,
                asp_rat: (width as f32) / (height as f32),
            })
        } else {
            Err(Box::new(WinitError::CreationError()))
        }
    }

    pub fn should_close(&self) -> bool {
        self.window.should_close()
    }

    pub fn close(&mut self) {
        self.window.set_should_close(true);
    }

    fn default_events_processor(&mut self) {
        self.glfw.poll_events();
        for (_, event) in glfw::flush_messages(&self.events) {
            match event {
                WindowEvent::FramebufferSize(width, height) => {
                    self.asp_rat = (width as f32) / (height as f32);
                    unsafe { gl::Viewport(0, 0, width, height) }
                }
                WindowEvent::Key(key, scancode, action, mode) => {
                    self.keyboard.key_callback(key, scancode, action, mode);
                }
                _ => {}
            }
        }
    }

    pub fn process_events(&mut self) {
        (self.events_processor)(self);
    }

    pub fn set_events_processor(&mut self, processor: fn(&mut Self)) {
        self.events_processor = processor;
    }

    pub fn swap_buffers(&mut self) {
        self.window.swap_buffers();
    }
}