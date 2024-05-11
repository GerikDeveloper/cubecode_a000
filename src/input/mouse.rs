use glfw::CursorMode;
use crate::render::types::Vec2d;
use crate::window::Window;

pub struct Mouse {
    pos: Vec2d,
    delta_pos: Vec2d,
    cursor_state: bool,
    is_started: bool,
}

impl Mouse {

    pub fn new() -> Self {
        return Self {
            pos: [0.0f64, 0.0f64],
            delta_pos: [0.0f64, 0.0f64],
            cursor_state: true,
            is_started: false,
        }
    }
    pub fn cursor_pos_callback(&mut self, xpos: f64, ypos: f64) {
        if self.is_started {
            self.delta_pos = [(xpos - self.pos[0]), (ypos - self.pos[1])];
        } else {self.is_started = true;}
        self.pos = [xpos, ypos];
    }

    pub fn set_cursor_state(&mut self, window: &Window, active: bool)  {
        self.cursor_state = active;
        if active {
            window.set_cursor_mode(CursorMode::Normal);
        } else {
            window.set_cursor_mode(CursorMode::Disabled);
        }
    }

    pub fn get_cursor_state(&self) -> bool {
        return self.cursor_state;
    }

    pub fn get_cursor_pos(&self) -> &Vec2d {
        return &self.pos;
    }

    pub fn get_cursor_delta_pos(&self) -> &Vec2d {
        return &self.delta_pos;
    }

    pub fn toggle_cursor(&mut self, window: &Window) {
        self.set_cursor_state(window, (!self.cursor_state));
    }

    pub fn poll_events(&mut self) {
        self.delta_pos = [0.0, 0.0];
    }
}