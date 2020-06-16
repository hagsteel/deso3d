use gdnative::{Control, GodotObject, KinematicBody, PackedScene, ResourceLoader, TextureRect};

pub fn spawn_unit() -> KinematicBody {
    load_resource("res://characters/humanoid_a.tscn")
}

pub fn spawn_enemy() -> KinematicBody {
    load_resource("res://BadGuy.tscn")
}

fn load_resource<T: GodotObject>(path: &str) -> T {
    let mut loader = ResourceLoader::godot_singleton();
    loader
        .load(path.into(), "PackedScene".into(), false)
        .and_then(|res| res.cast::<PackedScene>())
        .and_then(|scn| scn.instance(0))
        .and_then(|nde| unsafe { nde.cast::<T>() })
        .unwrap()
}

pub fn spawn_formation_ui() -> TextureRect {
    load_resource("res://FormationUI.tscn")
}

pub fn spawn_formation_unit() -> TextureRect {
    load_resource("res://FormationUnit.tscn")
}

pub fn spawn_context_menu() -> Control {
    let mut context_menu = load_resource::<Control>("res://ContextMenu.tscn");
    unsafe { context_menu.set_visible(false) };
    context_menu
}
