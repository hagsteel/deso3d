use gdnative::*;

// mod game;
mod gameworld;
mod input;
mod movement;
mod unit;
mod camera;
mod spawner;
mod tilemap;
mod procgen;
mod player;
mod saveload;
mod enemy;
// mod main_menu;
mod formation;
mod animation;
// // mod dragndrop;
mod debug;
mod contextmenu;

fn init(handle: init::InitHandle) {
    handle.add_class::<gameworld::GameWorld>();
    // handle.add_class::<main_menu::MainMenu>();
    handle.add_class::<debug::DebugDraw>();
    handle.add_class::<contextmenu::ContextMenu>();
}

godot_gdnative_init!();
godot_nativescript_init!(init);
godot_gdnative_terminate!();

#[macro_export]
macro_rules! safe {
    ($var:ident) => {
        let $var = unsafe { $var.assume_safe() };
    }
}

#[cfg(feature = "godot_test")]
macro_rules! run_test {
    ($test:expr) => {
        if $test() {
            println!("{} [Ok]", stringify!($test));
            true
        } else {
            println!("{} [Failed]", stringify!($test));
            false
        }
    }
}

#[cfg(feature = "godot_test")]
#[macro_export]
macro_rules! assert_gd {
    ($assert_exp:expr) => {
        if !$assert_exp {
            let line = std::line!();
            let file = std::file!();
            eprintln!("{}: {}", file, line);
            return false
        } else {
            true
        }
    }
}

#[no_mangle]
#[cfg(feature = "godot_test")]
pub extern fn run_tests() -> sys::godot_variant {
    let mut status = true;

    eprintln!("Running tests: [add your tests here]");
    // status &= run_test!(path::to::test_fn);

    gdnative::Variant::from_bool(status).forget()
}
