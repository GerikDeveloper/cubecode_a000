use std::cell::RefCell;
use std::sync::mpsc::Receiver;
use glfw::{Action, Context, CursorMode, Glfw, Key, WindowEvent};
use glfw::ffi::{glfwGetTime, glfwSetInputMode};
use thiserror::Error;
use crate::input::keyboard::Keyboard;
use crate::input::mouse::Mouse;
use crate::render::types::Vec2ui;

#[derive(Error, Debug)]
pub enum WinitError {
    #[error("Window creation failed")]
    CreationError(),
}

pub struct Window {
    pub glfw: Glfw,
    pub window: RefCell<glfw::Window>,
    width: u32,
    height: u32,
    pub events: Receiver<(f64, WindowEvent)>,
    pub events_processor: fn(&mut Window),
    pub keyboard: Keyboard,
    pub mouse: RefCell<Mouse>,
    pub asp_rat: f32,
}

impl Window {
    pub fn init(width: u32, height: u32, title: &str, keyboard: Keyboard, mouse: Mouse) -> Result<Self, Box<dyn std::error::Error>> {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS)?;
        glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
        //#[cfg(target_os = "macos")]
        glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
        if let Some((mut window, mut events)) = glfw.create_window(width, height, title, glfw::WindowMode::Windowed) {
            window.make_current();
            window.set_key_polling(true);
            window.set_cursor_pos_polling(true);
            window.set_mouse_button_polling(true);
            window.set_framebuffer_size_polling(true);
            gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
            Ok(Self {
                glfw,
                window: RefCell::from(window),
                width,
                height,
                events,
                events_processor: |wnd| Self::default_events_processor(wnd),
                keyboard,
                mouse: RefCell::from(mouse),
                asp_rat: (width as f32) / (height as f32),
            })
        } else {
            Err(Box::new(WinitError::CreationError()))
        }
    }

    pub fn should_close(&self) -> bool {
        self.window.borrow().should_close()
    }

    pub fn close(&self) {
        self.window.borrow_mut().set_should_close(true);
    }

    fn default_events_processor(&mut self) {
        self.glfw.poll_events();
        self.mouse.borrow_mut().poll_events();
        for (_, event) in glfw::flush_messages(&self.events) {
            match event {
                WindowEvent::FramebufferSize(width, height) => {
                    self.width = (width as u32);
                    self.height = (height as u32);
                    self.asp_rat = (width as f32) / (height as f32);
                    unsafe { gl::Viewport(0, 0, width, height) }
                }
                WindowEvent::Key(key, scancode, action, mode) => {
                    self.keyboard.key_callback(key, scancode, action, mode);
                }
                WindowEvent::CursorPos(xpos, ypos) => {
                    self.mouse.borrow_mut().cursor_pos_callback(xpos, ypos);
                }
                WindowEvent::MouseButton(button, action, modifiers) => {
                    self.mouse.borrow_mut().button_callback(button, action, modifiers);
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

    pub fn swap_buffers(&self) {
        self.window.borrow_mut().swap_buffers();
    }

    pub fn set_cursor_mode(&self, mode: CursorMode) {
        unsafe {
            self.window.borrow_mut().set_cursor_mode(mode);
        }
    }

    pub fn get_width(&self) -> u32 {
        return self.width;
    }

    pub fn get_height(&self) -> u32 {
        return self.height;
    }

    pub fn get_size(&self) -> Vec2ui {
        return [self.width, self.height];
    }

    pub fn get_time(&self) -> f64 {
        unsafe { return glfwGetTime(); }
    }
}