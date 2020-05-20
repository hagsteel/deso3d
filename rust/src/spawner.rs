use gdnative::{GodotObject, KinematicBody, PackedScene, ResourceLoader, TextureRect};

pub fn spawn_unit() -> KinematicBody {
    load_resource("res://Unit.tscn")
}

pub fn spawn_enemy() -> KinematicBody {
    load_resource("res://BadGuy.tscn")
}

fn load_resource<T: GodotObject>(path: &str) -> T {
    let mut loader = ResourceLoader::godot_singleton();
    loader
        .load(path.into(), "PackedScene".into(), false)
        .and_then(|mut res| res.cast::<PackedScene>())
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

