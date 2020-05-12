use gdnative::{InputEventMouseButton, Vector2};
use bitflags::bitflags;

pub const LMB: i64 = 1;
pub const RMB: i64 = 2;

pub enum MouseButton {
    Empty,
    Mouse { pressed: bool, button_index: i64 },
}

impl MouseButton {
    pub fn consume(&mut self) {
        *self = Self::Empty;
    }

    pub fn from_event(ev: InputEventMouseButton) -> Self {
        Self::Mouse {
            pressed: ev.is_pressed(),
            button_index: ev.get_button_index(),
        }
    }

    pub fn button_pressed(&self, index: i64) -> bool {
        match self {
            Self::Empty => false,
            Self::Mouse { pressed, button_index } => {
                *pressed && *button_index == index
            }
        }
    }
}

pub struct MousePos {
    global: Vector2,
}

impl MousePos {
    pub fn set_global(&mut self, pos: Vector2) {
        self.global = pos;
    }

    pub fn global(&self) -> Vector2 {
        self.global
    }

    pub fn zero() -> Self {
        Self {
            global: Vector2::zero(),
        }
    }
}

bitflags! {
    pub struct Keys: u32 {
        const Empty = 0;
        const Left = 1;
        const Right = 2;
        const Up = 4;
        const Down = 8;
    }
}

pub struct Keyboard {
    // is_pressed: bool,
    keys: Keys,
}

impl Keyboard {
    pub fn new() -> Self {
        Self {
            // is_pressed: false,
            keys: Keys::Empty,
        }
    }

    pub fn set_empty(&mut self) {
        self.keys = Keys::Empty;
    }

    pub fn update(&mut self, key: Keys, is_pressed: bool) {
        // self.is_pressed = is_pressed;
        match is_pressed {
            true => self.keys |= key,
            false => self.keys.remove(key),
        }
    }

    pub fn keys(&self) -> Keys {
        self.keys
    }
}
