use gdnative::{
    godot_error, godot_wrap_method, godot_wrap_method_inner, godot_wrap_method_parameter_count,
    methods, Control, InputEvent, InputEventMouse, NativeClass,
};

#[derive(NativeClass)]
#[inherit(Control)]
pub struct MainMenu {}

#[methods]
impl MainMenu {
    pub fn _init(_owner: Control) -> Self {
        Self {}
    }

    #[export]
    pub fn _ready(&self, owner: Control) {
        eprintln!("hello");
    }

    #[export]
    pub fn new_game(&self, owner: Control, event: InputEvent) {
        if let Some(ev) = event.cast::<InputEventMouse>() {
            if ev.is_pressed() {
                eprintln!("{:?}", "new game selected");
            }
        }
    }

    #[export]
    pub fn load_game(&mut self, owner: Control, event: InputEvent, slot: i64) {
        if let Some(ev) = event.cast::<InputEventMouse>() {
            if ev.is_pressed() {
                eprintln!("Load game {}", slot);
            }
        }
    }
}
