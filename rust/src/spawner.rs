use gdnative::{KinematicBody, ResourceLoader, GodotObject, PackedScene};

pub fn spawn_unit() -> KinematicBody {
    load_resource("res://Unit.tscn")
}

fn load_resource<T: GodotObject>(path: &str) -> T {
    let mut loader = ResourceLoader::godot_singleton();
    loader.load(path.into(), "PackedScene".into(), false)
        .and_then(|res| res.cast::<PackedScene>())
        .and_then(|scn| scn.instance(0) )
        .and_then(|nde| unsafe { nde.cast::<T>() })
        .unwrap()
}
