use gdextras::node_ext::NodeExt;
use gdextras::some_or_bail;
// use gdnative::{
//     godot_error, godot_wrap_method, godot_wrap_method_inner, godot_wrap_method_parameter_count,
//     methods, Color, InputEvent, NativeClass, Node2D, Vector2, Vector3, Camera,
// };

use gdnative::{
    godot_error, godot_wrap_method, godot_wrap_method_inner, godot_wrap_method_parameter_count,
    methods, NativeClass
};
use gdnative::api::{InputEvent, Node2D, Camera};

use crate::gameworld::Line;

#[derive(NativeClass)]
#[inherit(Node2D)]
pub struct DebugDraw {
    lines: Vec<Line>,
}

#[methods]
impl DebugDraw {
    pub fn _init(_: &Node2D) -> Self {
        Self { lines: Vec::new() }
    }

    pub fn set_lines(&mut self, mut lines: Vec<Line>) {
        self.lines.append(&mut lines);
    }

    #[export]
    pub fn _draw(&mut self, owner: &Node2D) {
        while let Some(line) = self.lines.pop() {
            let camera = owner.get_and_cast::<Camera>("../Camera");
            let start = camera.unproject_position(line.0);
            let end = camera.unproject_position(line.1);
            let col = line.2;
            let thickness = line.3;

            owner.draw_line(start, end, col, thickness, false);
        }
    }
}
