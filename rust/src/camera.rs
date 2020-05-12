use gdnative::{
    Camera as GodotCamera, MeshInstance, PhysicsServer, Variant, VariantArray, Vector2, Vector3,
    Area
};
use legion::prelude::*;
use legion::systems::schedule::Builder;

use crate::gameworld::Delta;
use crate::input::{Keyboard, Keys};

pub const RAY_LENGTH: f32 = 1000.;
const CAMERA_SPEED: f32 = 80.;

// -----------------------------------------------------------------------------
//     - Resources -
// -----------------------------------------------------------------------------
#[derive(Debug, Clone, Copy)]
pub enum Drag {
    Empty,
    Start(Vector3),
}

pub struct SelectionBox(pub MeshInstance);

unsafe impl Send for SelectionBox {}
unsafe impl Sync for SelectionBox {}

pub struct UnitSelectionArea(pub Area);

unsafe impl Send for UnitSelectionArea {}
unsafe impl Sync for UnitSelectionArea {}

impl Drag {
    pub fn set_start(&mut self, pos: Vector3) {
        *self = Self::Start(pos);
    }

    pub fn clear(&mut self) {
        *self = Self::Empty;
    }
}

pub struct Camera(pub GodotCamera);

unsafe impl Send for Camera {}
unsafe impl Sync for Camera {}

impl Camera {
    pub fn pos_from_camera(&self, mouse_pos: Vector2, ray_length: f32) -> Option<Vector3> {
        let from = unsafe { self.0.project_ray_origin(mouse_pos) };
        let to = from + unsafe { self.0.project_ray_normal(mouse_pos) } * ray_length;

        let rid = unsafe {
            let world = self.0.get_world().expect("failed to get world");
            world.get_space()
        };

        let mut phys_server = PhysicsServer::godot_singleton();
        let mut direct_state = phys_server
            .space_get_direct_state(rid)
            .expect("failed to get direct state");

        unsafe {
            let dict = direct_state.intersect_ray(
                from,                // From
                to,                  // To
                VariantArray::new(), // Ignored objects
                2,                   // Collision mask
                false,               // Collide with bodies
                true,                // Collide with areas
            );
            let pos_variant = dict.get(&Variant::from_godot_string(&"position".into()));
            if pos_variant.is_nil() {
                return None;
            }
            let pos = pos_variant.to_vector3();
            Some(pos)
        }
    }
}

// -----------------------------------------------------------------------------
//     - Systems -
// -----------------------------------------------------------------------------

fn move_camera() -> Box<dyn Runnable> {
    SystemBuilder::new("move camera")
        .read_resource::<Keyboard>()
        .write_resource::<Camera>()
        .write_resource::<UnitSelectionArea>()
        .read_resource::<Delta>()
        .build_thread_local(|_, _, res, _| {
            let (keyboard, camera, unit_sel_area, delta) = res;

            if keyboard.keys() == Keys::Empty {
                return;
            }

            let mut translation = Vector3::zero();

            if keyboard.keys() & Keys::Left == Keys::Left {
                translation.x += -1.0;
            }

            if keyboard.keys() & Keys::Right == Keys::Right {
                translation.x += 1.0;
            }

            if keyboard.keys() & Keys::Up == Keys::Up {
                translation.z += -1.0;
            }

            if keyboard.keys() & Keys::Down == Keys::Down {
                translation.z += 1.0;
            }

            unsafe {
                let current_translation = camera.0.get_translation();
                translation *= CAMERA_SPEED * delta.0;
                camera.0.set_translation(current_translation + translation);
                let mut camera_x_z = camera.0.get_translation();
                camera_x_z.y = 0.;
                unit_sel_area.0.set_translation(camera_x_z);
            }
        })
}

pub fn camera_systems(builder: Builder) -> Builder {
    builder.add_thread_local(move_camera())
}
