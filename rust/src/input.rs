use gdnative::{InputEventMouseButton, Vector2};

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
