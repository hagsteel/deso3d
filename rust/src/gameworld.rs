use gdextras::input::InputEventExt;
use gdextras::node_ext::NodeExt;
use gdnative::{
    godot_error, godot_wrap_method, godot_wrap_method_inner, godot_wrap_method_parameter_count,
    methods, Camera as GodotCamera, InputEvent, InputEventMouse, InputEventMouseButton,
    KinematicBody, NativeClass, Spatial, Vector3,
};
use legion::prelude::*;

use crate::camera::{mouse_cam, Camera};
use crate::input::{MouseButton, MousePos};
use crate::movement::{
    apply_directional_velocity, apply_gravity, done_moving, move_units, rotate_unit, Pos, Speed,
    Velocity,
};
use crate::spawner;
use crate::unit::Unit;

fn setup_schedule() -> Schedule {
    Schedule::builder()
        .add_thread_local(apply_directional_velocity())
        .add_thread_local(apply_gravity())
        .add_thread_local(rotate_unit())
        .add_thread_local(move_units())
        .add_thread_local(done_moving())
        .add_thread_local(mouse_cam())
        .build()
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
}

#[methods]
impl GameWorld {
    pub fn _init(_owner: Spatial) -> Self {
        let physics = setup_schedule();
        let mut resources = Resources::default();
        resources.insert(Delta(0.));
        resources.insert(MouseButton::Empty);
        resources.insert(MousePos::zero());

        Self {
            world: Universe::new().create_world(),
            resources,
            physics,
        }
    }

    #[export]
    pub fn _ready(&mut self, mut owner: Spatial) {
        // Camera
        let camera = owner
            .get_and_cast::<GodotCamera>("Camera")
            .expect("failed to get camera");
        self.resources.insert(Camera(camera));

        // Player unit
        for x in 0..10 {
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
                (),
                Some((Unit::new(unit), Velocity(Vector3::zero()), speed, Pos(pos))),
            );
        }
    }

    #[export]
    pub fn _unhandled_input(&mut self, owner: Spatial, event: InputEvent) {
        if event.action_pressed("ui_cancel") {
            unsafe { owner.get_tree().map(|mut tree| tree.quit(0)) };
        }

        if let Some(btn_event) = event.cast::<InputEventMouseButton>() {
            self.resources.get_mut::<MouseButton>().map(|mut btn| {
                *btn = MouseButton::from_event(btn_event);
            });
        }

        if let Some(mouse_event) = event.cast::<InputEventMouse>() {
            self.resources.get_mut::<MousePos>().map(|mut pos| {
                pos.set_global(mouse_event.get_global_position());
            });
        }
    }

    #[export]
    pub fn _physics_process(&mut self, _: Spatial, delta: f64) {
        self.resources
            .get_mut::<Delta>()
            .map(|mut d| d.0 = delta as f32);
        self.physics.execute(&mut self.world, &mut self.resources);
    }
}
