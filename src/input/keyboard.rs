use std::cell::Cell;
use glfw::{Action, Key, Modifiers, Scancode};

pub struct Keyboard {
    keys: Vec<Cell<bool>>,
}

impl Keyboard {

    pub fn new() -> Self {
        let mut keys: Vec<Cell<bool>> = Vec::new();
        for key in 0..1024 {
            keys.push(Cell::from(false));
        }

        Keyboard { keys }
    }

    pub fn key_callback(&self, key: Key, scancode: Scancode, action: Action, mode: Modifiers) {
        if action == Action::Press {
            if let Some(state) = self.keys.get(key as usize) {
                state.set(true);
            }
        }
        if action == Action::Release {
            if let Some(state) = self.keys.get(key as usize) {
                state.set(false);
            }
        }
    }

    pub fn get_key_state(&self, key: Key) -> bool {
        return if let Some(state) = self.keys.get(key as usize) {
            state.get()
        } else {
            false
        }
    }
}