use gdextras::input::InputEventExt;
use gdextras::node_ext::NodeExt;
use gdnative::{
    godot_error, godot_wrap_method, godot_wrap_method_inner, godot_wrap_method_parameter_count,
    methods, Area, Camera as GodotCamera, GridMap, InputEvent, InputEventKey, InputEventMouse,
    InputEventMouseButton, Label, MeshInstance, NativeClass, Performance, Spatial, Vector3,
};
use lazy_static::lazy_static;
use legion::prelude::*;
use std::sync::Mutex;

use crate::camera::{camera_systems, Camera, Drag, SelectionBox, UnitSelectionArea};
use crate::enemy::{enemy_systems, DetectionRange, Enemy};
use crate::input::{Keyboard, Keys, MouseButton, MousePos};
use crate::movement::{movement_systems, Pos, MaxSpeed, Velocity, Forces, Acceleration};
use crate::player::{player_systems, PlayerId};
use crate::saveload;
use crate::spawner;
use crate::tilemap::{draw_tilemap, Coords, TileMap};
use crate::unit::Unit;

fn setup_physics_schedule() -> Schedule {
    let builder = Schedule::builder();
    let builder = movement_systems(builder);

    builder.build()
}

fn setup_schedule() -> Schedule {
    let builder = Schedule::builder().add_thread_local(draw_tilemap());
    let builder = enemy_systems(builder);
    let builder = player_systems(builder);
    let builder = camera_systems(builder);
    builder.build()
}

// -----------------------------------------------------------------------------
//     - World -
// -----------------------------------------------------------------------------

lazy_static! {
    static ref WORLD: Mutex<World> = Mutex::new(Universe::new().create_world());
}

pub fn with_world<F>(f: F)
where
    F: FnOnce(&mut World),
{
    let _ = WORLD.try_lock().map(|mut world| f(&mut world));
}

// -----------------------------------------------------------------------------
//     - Resources -
// -----------------------------------------------------------------------------
pub struct Delta(pub f32);

pub struct A(pub f64);
pub struct B(pub f64);

// -----------------------------------------------------------------------------
//     - Godot node -
// -----------------------------------------------------------------------------

#[derive(NativeClass)]
#[inherit(Spatial)]
pub struct GameWorld {
    resources: Resources,
    physics: Schedule,
    process: Schedule,
}

#[methods]
impl GameWorld {
    pub fn _init(_owner: Spatial) -> Self {
        let physics = setup_physics_schedule();
        let process = setup_schedule();
        let mut resources = Resources::default();
        resources.insert(Delta(0.));
        resources.insert(MouseButton::Empty);
        resources.insert(MousePos::zero());
        resources.insert(Coords::new());
        resources.insert(Keyboard::new());
        resources.insert(Drag::Empty);
        resources.insert(A(1.0));
        resources.insert(B(1.0));

        Self {
            resources,
            physics,
            process,
        }
    }

    #[export]
    pub fn _ready(&mut self, mut owner: Spatial) {
        // Tilemap
        let gridmap = owner
            .get_and_cast::<GridMap>("GridMap")
            .expect("failed to get grid map");
        self.resources.insert(TileMap(gridmap));

        // Camera
        let camera = owner
            .get_and_cast::<GodotCamera>("Camera")
            .expect("failed to get camera");
        self.resources.insert(Camera(camera));

        // Unit selection area (detect mouse selection)
        let unit_selection_area = owner
            .get_and_cast::<Area>("UnitSelectionArea")
            .expect("failed to get unit selection area");
        self.resources
            .insert(UnitSelectionArea(unit_selection_area));

        // Draw selection node
        let selection_box = owner
            .get_and_cast::<MeshInstance>("SelectionBox")
            .expect("failed to get selection box");
        self.resources.insert(SelectionBox(selection_box));

        // Player unit
        for x in 15..19 {
            let x = x as f32 * 4.;
            let y = 2.;
            let z = 10.;

            let mut unit = spawner::spawn_unit();
            unsafe {
                owner.add_child(Some(unit.to_node()), false);
                unit.set_translation(Vector3::new(x, y, z));
            }

            let pos = unsafe { unit.get_translation() };

            let speed = MaxSpeed(10f32);

            with_world(|world| {
                world.insert(
                    (PlayerId::new(x as u8),),
                    Some((
                        Unit::new(unit),
                        Velocity(Vector3::zero()),
                        speed,
                        Pos(pos),
                        Forces::zero(),
                        Acceleration(Vector3::zero())
                    )),
                );
            });
        }

        for x in 15..15 {
            let x = x as f32 * 4.;
            let y = 12.;
            let z = 26.;

            let mut unit = spawner::spawn_enemy();
            unsafe {
                owner.add_child(Some(unit.to_node()), false);
                unit.set_translation(Vector3::new(x, y, z));
            }

            let pos = unsafe { unit.get_translation() };

            let speed = MaxSpeed(10f32);

            with_world(|world| {
                world.insert(
                    (Enemy,),
                    Some((
                        Unit::new(unit),
                        Velocity(Vector3::zero()),
                        speed,
                        Pos(pos),
                        DetectionRange(10.),
                        Forces::zero(),
                        Acceleration(Vector3::zero())
                    )),
                );
            });
        }
    }

    #[export]
    pub fn _unhandled_input(&mut self, owner: Spatial, event: InputEvent) {
        if event.action_pressed("ui_cancel") {
            unsafe { owner.get_tree().map(|mut tree| tree.quit(0)) };
        }

        if event.action_pressed("save") {
            if let Err(e) = saveload::save(0) {
                eprintln!("{:?}", e);
            }
        }

        // Mouse button
        if let Some(btn_event) = event.cast::<InputEventMouseButton>() {
            self.resources.get_mut::<MouseButton>().map(|mut btn| {
                *btn = MouseButton::from_event(btn_event);
            });
        }

        // Mouse pos
        if let Some(mouse_event) = event.cast::<InputEventMouse>() {
            self.resources.get_mut::<MousePos>().map(|mut pos| {
                pos.set_global(mouse_event.get_global_position());
            });
        }

        // Keyboard
        if let Some(_) = event.cast::<InputEventKey>() {
            self.resources.get_mut::<Keyboard>().map(|mut key| {
                if event.is_action_pressed("Left".into(), false) {
                    key.update(Keys::LEFT, true);
                } else if event.is_action_released("Left".into()) {
                    key.update(Keys::LEFT, false);
                }

                if event.is_action_pressed("Right".into(), false) {
                    key.update(Keys::RIGHT, true);
                } else if event.is_action_released("Right".into()) {
                    key.update(Keys::RIGHT, false);
                }

                if event.is_action_pressed("Up".into(), false) {
                    key.update(Keys::UP, true);
                } else if event.is_action_released("Up".into()) {
                    key.update(Keys::UP, false);
                }

                if event.is_action_pressed("Down".into(), false) {
                    key.update(Keys::DOWN, true);
                } else if event.is_action_released("Down".into()) {
                    key.update(Keys::DOWN, false);
                }
            });
        }
    }

    #[export]
    pub fn _process(&mut self, owner: Spatial, _: f64) {
        with_world(|world| {
            self.process.execute(world, &mut self.resources);
        });

        // Debug label
        let mut label = owner.get_and_cast::<Label>("UI/Panel/DebugLabel").unwrap();
        let perf = Performance::godot_singleton();
        let fps = format!("fps: {}", perf.get_monitor(Performance::TIME_FPS));
        unsafe { label.set_text(fps.into()) };
    }

    #[export]
    pub fn _physics_process(&mut self, _: Spatial, delta: f64) {
        self.resources
            .get_mut::<Delta>()
            .map(|mut d| d.0 = delta as f32);
        with_world(|world| {
            self.physics.execute(world, &mut self.resources);
        });
    }

    #[export]
    pub fn a_value_changed(&mut self, _: Spatial, value: f64) {
        self.resources.get_mut::<A>().map(|mut a| a.0 = value);
    }

    #[export]
    pub fn b_value_changed(&self, _: Spatial, value: f64) {
        self.resources.get_mut::<B>().map(|mut b| b.0 = value);
    }
}
