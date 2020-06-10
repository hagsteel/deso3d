use gdnative::{
    Area, Camera as GodotCamera, MeshInstance, PhysicsServer, Variant, VariantArray, Vector2,
    Vector3,
};
use legion::prelude::*;
use legion::systems::schedule::Builder;

use crate::gameworld::{ClickIndicator, Delta};
use crate::input::{Keyboard, Keys, MousePos, MouseButton, RMB};

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
    pub fn pos_from_camera(
        &self,
        mouse_pos: Vector2,
        ray_length: f32,
        col_mask: i64,
    ) -> Option<Vector3> {
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
                col_mask,            // Collision mask
                true,                // Collide with bodies
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

            if keyboard.keys() == Keys::EMPTY {
                return;
            }

            let mut translation = Vector3::zero();

            if keyboard.keys() & Keys::LEFT == Keys::LEFT {
                translation.x += -1.0;
            }

            if keyboard.keys() & Keys::RIGHT == Keys::RIGHT {
                translation.x += 1.0;
            }

            if keyboard.keys() & Keys::UP == Keys::UP {
                translation.z += -1.0;
            }

            if keyboard.keys() & Keys::DOWN == Keys::DOWN {
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

fn set_click_indicator() -> Box<dyn Runnable> {
    SystemBuilder::new("set click indicator")
        .read_resource::<Camera>()
        .write_resource::<ClickIndicator>()
        .read_resource::<MouseButton>()
        .read_resource::<MousePos>()
        .build_thread_local(|_cmd, _world, resources, _query| {
            let (camera, click_indicator, mouse_btn, mouse_pos) = resources;
            if !mouse_btn.button_pressed(RMB) {
                return;
            }

            let dest_pos = match camera.pos_from_camera(mouse_pos.global(), RAY_LENGTH, 1) {
                Some(p) => p,
                None => return,
            };

            click_indicator.set_position(dest_pos);
        })
}

pub fn camera_systems(builder: Builder) -> Builder {
    builder
        .add_thread_local(move_camera())
        .add_thread_local(set_click_indicator())
}
