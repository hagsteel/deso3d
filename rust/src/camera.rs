use gdextras::camera::CameraExt;
use gdnative::{Camera as GodotCamera, MeshInstance, Rect2, Vector2, Vector3};
use legion::prelude::*;

use crate::gameworld::Delta;
use crate::input::{Keyboard, Keys, MouseButton, MousePos};
use crate::movement::Destination;
use crate::movement::Pos;
use crate::unit::Player;

const RAY_LENGTH: f32 = 1000.;
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
    fn set_start(&mut self, pos: Vector3) {
        *self = Self::Start(pos);
    }

    fn clear(&mut self) {
        *self = Self::Empty;
    }
}

// -----------------------------------------------------------------------------
//     - Components -
// -----------------------------------------------------------------------------
pub struct Camera(pub GodotCamera);

unsafe impl Send for Camera {}
unsafe impl Sync for Camera {}

// TODO:
// 1.  Mark a unit as "selected"

// -----------------------------------------------------------------------------
//     - Systems -
// -----------------------------------------------------------------------------
pub fn select_position() -> Box<dyn Runnable> {
    SystemBuilder::new("mouse camera doda")
        .read_resource::<MouseButton>()
        .read_resource::<MousePos>()
        .read_resource::<Camera>()
        .write_resource::<SelectionBox>()
        .write_resource::<Drag>()
        .with_query(<Read<Pos>>::query().filter(tag::<Player>()))
        .build_thread_local(|cmd, world, resources, unit_positions| {
            let (mouse_btn, mouse_pos, camera, selection_box, drag) = resources;

            let mut pos = match camera.0.pos_from_camera(mouse_pos.global(), RAY_LENGTH) {
                Some(p) => p,
                None => return,
            };

            if !mouse_btn.button_pressed(1) {
                if let Drag::Start(start_pos) = drag as &mut Drag {
                    // Selection
                    let start_2d = Vector2::new(start_pos.x, start_pos.z).to_point();
                    let end_2d = Vector2::new(pos.x, pos.z).to_point();
                    let size = (start_2d - end_2d).abs();
                    let point = Vector2::new(start_2d.x.min(end_2d.x), start_2d.y.min(end_2d.y));
                    let selection = Rect2::new(point.to_point(), size.to_size());

                    for unit_pos in unit_positions.iter(world) {
                        let unit_pos_2d = Vector2::new(unit_pos.0.x, unit_pos.0.z);
                        if selection.contains(unit_pos_2d.to_point()) {
                            eprintln!("Selected unit");
                        }
                    }
                }

                unsafe { selection_box.0.set_scale(Vector3::zero()) };
                drag.clear();
                return;
            }

            pos.y = 1.0;

            match drag as &mut Drag {
                Drag::Empty => {
                    drag.set_start(pos);
                    unsafe { selection_box.0.set_translation(pos) };
                }
                Drag::Start(start_pos) => {
                    let mut size = pos - *start_pos;
                    size.y = 0.3;
                    unsafe {
                        selection_box.0.set_scale(size);
                        let translation = pos - size / 2.;
                        selection_box.0.set_translation(translation);
                    }

                }
            }
        })
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
