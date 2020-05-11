use gdnative::{Camera as GodotCamera, PhysicsServer, Variant, VariantArray};
use legion::prelude::*;

use crate::input::{MouseButton, MousePos};
use crate::movement::Destination;
use crate::unit::Unit;

const RAY_LENGTH: f32 = 1000.;

pub struct Camera(pub GodotCamera);

unsafe impl Send for Camera {}
unsafe impl Sync for Camera {}

pub fn mouse_cam() -> Box<dyn Runnable> {
    SystemBuilder::new("mosue camera doda")
        .read_resource::<MouseButton>()
        .read_resource::<MousePos>()
        .read_resource::<Camera>()
        .with_query(<Read<Unit>>::query())
        .build_thread_local(|cmd, world, (mouse_btn, mouse_pos, camera), units| {
            if !mouse_btn.button_pressed(1) {
                return;
            }

            let pos = mouse_pos.global();
            let from = unsafe { camera.0.project_ray_origin(pos) };
            let to = from + unsafe { camera.0.project_ray_normal(pos) } * RAY_LENGTH;

            let rid = unsafe {
                let world = camera.0.get_world().expect("failed to get world");
                world.get_space()
            };

            let mut phys_server = PhysicsServer::godot_singleton();
            let mut direct_state = phys_server
                .space_get_direct_state(rid)
                .expect("failed to get direct state");

            let pos = unsafe {
                let dict = direct_state.intersect_ray(
                    from,                // From
                    to,                  // To
                    VariantArray::new(), // Ignored objects
                    1,                   // Collision mask
                    true,                // Collide with bodies
                    false,               // Collide with areas
                );
                let pos_variant = dict.get(&Variant::from_godot_string(&"position".into()));
                if pos_variant.is_nil() {
                    return
                }
                let mut pos = pos_variant.to_vector3();
                pos.y = 0.0;
                pos
            };

            for (entity, _) in units.iter_entities(world) {
                cmd.add_component(entity, Destination(pos));
            }
        })
}
