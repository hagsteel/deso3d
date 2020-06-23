use gdnative::api::{Control, KinematicBody, PackedScene, ResourceLoader, TextureRect, Node};
use gdnative::{GodotObject, Ptr};

pub fn spawn_unit() -> Ptr<KinematicBody> {
    unsafe { 
        load_resource("res://characters/humanoid_a.tscn")
        .assume_safe()
        .cast::<KinematicBody>()
        .unwrap()
        .claim()
    }
}

pub fn spawn_enemy() -> Ptr<KinematicBody> {
    unsafe { load_resource("res://BadGuy.tscn")
        .assume_safe()
        .cast::<KinematicBody>()
        .unwrap()
        .claim()
    }
}

// fn load_resource<T: ManuallyManaged>(path: &str) -> T {
fn load_resource(path: &str) -> Ptr<Node> {
    let loader = ResourceLoader::godot_singleton();
    loader
        .load(path.into(), "PackedScene".into(), false)
        .and_then(|res| res.cast::<PackedScene>())
        .and_then(|scn| scn.instance(0))
        .unwrap()
        // .and_then(|nde| nde)
}

pub fn spawn_formation_ui() -> Ptr<TextureRect> {
    unsafe { 
        load_resource("res://FormationUI.tscn")
        .assume_safe()
        .cast::<TextureRect>()
        .unwrap()
        .claim()
    }
}

pub fn spawn_formation_unit() -> Ptr<TextureRect> {
    unsafe { 
        load_resource("res://FormationUnit.tscn")
        .assume_safe()
        .cast::<TextureRect>()
        .unwrap()
        .claim()
    }
}

pub fn spawn_context_menu() -> Ptr<Control> {
    let mut context_menu = unsafe { 
        load_resource("res://ContextMenu.tscn")
        .assume_safe()
        .cast::<Control>()
        .unwrap()
        .claim()
    };
    unsafe { context_menu.assume_safe().set_visible(false) };
    context_menu
}
