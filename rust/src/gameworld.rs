use gdextras::input::InputEventExt;
use gdextras::node_ext::NodeExt;
use gdnative::{
    godot_error, godot_wrap_method, godot_wrap_method_inner, godot_wrap_method_parameter_count,
    methods, Camera as GodotCamera, GridMap, InputEvent, InputEventKey, InputEventMouse,
    InputEventMouseButton, Label, MeshInstance, NativeClass, Performance, Spatial, Vector3,
    Area
};
use legion::prelude::*;

use crate::camera::{camera_systems, Camera, Drag, SelectionBox, UnitSelectionArea};
use crate::input::{Keyboard, Keys, MouseButton, MousePos};
use crate::movement::{
    apply_directional_velocity, apply_gravity, done_moving, move_units, rotate_unit, Pos, Speed,
    Velocity,
};
use crate::spawner;
use crate::tilemap::{draw_tilemap, Coords, TileMap};
use crate::unit::Unit;
use crate::player::{Player, player_systems};

fn setup_physics_schedule() -> Schedule {
    let builder = Schedule::builder()
        .add_thread_local(apply_directional_velocity())
        .add_thread_local(apply_gravity())
        .add_thread_local(rotate_unit())
        .add_thread_local(move_units())
        .add_thread_local(done_moving());

    let builder = player_systems(builder);
    let builder = camera_systems(builder);

    builder.build()
}

fn setup_schedule() -> Schedule {
    let builder = Schedule::builder()
        .add_thread_local(draw_tilemap());

    builder.build()
}

// -----------------------------------------------------------------------------
//     - Resources -
// -----------------------------------------------------------------------------
pub struct Delta(pub f32);

// -----------------------------------------------------------------------------
//     - Godot node -
// -----------------------------------------------------------------------------

#[derive(NativeClass)]
#[inherit(Spatial)]
pub struct GameWorld {
    world: World,
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

        Self {
            world: Universe::new().create_world(),
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
        self.resources.insert(UnitSelectionArea(unit_selection_area));

        // Draw selection node
        let selection_box = owner
            .get_and_cast::<MeshInstance>("SelectionBox")
            .expect("failed to get selection box");
        self.resources.insert(SelectionBox(selection_box));

        // Player unit
        for x in 0..1 {
            let x = x as f32 * 4.;
            let y = 12.;
            let z = 0.;

            let mut unit = spawner::spawn_unit();
            unsafe {
                owner.add_child(Some(unit.to_node()), false);
                unit.set_translation(Vector3::new(x, y, z));
            }

            let pos = unsafe { unit.get_translation() };

            let speed = Speed(10f32);

            self.world.insert(
                (Player,),
                Some((Unit::new(unit), Velocity(Vector3::zero()), speed, Pos(pos))),
            );
        }
    }

    #[export]
    pub fn _unhandled_input(&mut self, owner: Spatial, event: InputEvent) {
        if event.action_pressed("ui_cancel") {
            unsafe { owner.get_tree().map(|mut tree| tree.quit(0)) };
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
        self.process.execute(&mut self.world, &mut self.resources);

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
        self.physics.execute(&mut self.world, &mut self.resources);
    }
}
