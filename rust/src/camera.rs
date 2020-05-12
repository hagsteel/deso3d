use gdextras::camera::CameraExt;
use gdextras::node_ext::NodeExt;
use gdnative::{
    Camera as GodotCamera, MeshInstance, PhysicsServer, Variant, VariantArray, Vector3,
    Vector2, Rect2
};
use legion::prelude::*;

use crate::gameworld::Delta;
use crate::input::{Keyboard, Keys, MouseButton, MousePos};
use crate::movement::Destination;
use crate::unit::Unit;

const RAY_LENGTH: f32 = 1000.;
const CAMERA_SPEED: f32 = 80.;

// -----------------------------------------------------------------------------
//     - Components -
// -----------------------------------------------------------------------------
pub struct Camera(pub GodotCamera);

unsafe impl Send for Camera {}
unsafe impl Sync for Camera {}

// TODO:
// 1.  Control camera with WASD [DONE]
// 2.  Mark a unit as "selected"
// 3.  Rename mouse cam [DONE]

// -----------------------------------------------------------------------------
//     - Systems -
// -----------------------------------------------------------------------------
pub fn select_position() -> Box<dyn Runnable> {
    SystemBuilder::new("mouse camera doda")
        .read_resource::<MouseButton>()
        .read_resource::<MousePos>()
        .read_resource::<Camera>()
        .with_query(<Read<Unit>>::query())
        .build_thread_local(
            |cmd, world, (mouse_btn, mouse_pos, camera), units| {
                if !mouse_btn.button_pressed(1) {
                    return;
                }

                let mut pos = match camera.0.pos_from_camera(mouse_pos.global(), RAY_LENGTH) {
                    Some(p) => p,
                    None => return,
                };

                pos.y = 1.;

                for (entity, _) in units.iter_entities(world) {
                    cmd.add_component(entity, Destination(pos));
                }
            },
        )
}

pub fn move_camera() -> Box<dyn Runnable> {
    SystemBuilder::new("move camera")
        .read_resource::<Keyboard>()
        .write_resource::<Camera>()
        .read_resource::<Delta>()
        .build_thread_local(|_, _, res, _| {
            let (keyboard, camera, delta) = res;

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
            }
        })
}
