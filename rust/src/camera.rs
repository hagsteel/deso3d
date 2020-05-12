use gdnative::{Camera as GodotCamera, MeshInstance, Vector3};
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

impl Drag {
    pub fn set_start(&mut self, pos: Vector3) {
        *self = Self::Start(pos);
    }

    pub fn clear(&mut self) {
        *self = Self::Empty;
    }
}

// -----------------------------------------------------------------------------
//     - Components -
// -----------------------------------------------------------------------------
pub struct Camera(pub GodotCamera);

unsafe impl Send for Camera {}
unsafe impl Sync for Camera {}

fn move_camera() -> Box<dyn Runnable> {
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

pub fn camera_systems(builder: Builder) -> Builder {
    builder
        .add_thread_local(move_camera())
}
