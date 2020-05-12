use gdnative::{Rect2, Vector2, Vector3};
use legion::prelude::*;
use legion::systems::schedule::Builder;

use crate::input::{MouseButton, MousePos};
use crate::movement::Pos;
use crate::unit::{Player, Selected};
use crate::camera::{Camera, SelectionBox, Drag, RAY_LENGTH};

// -----------------------------------------------------------------------------
//     - Systems -
// -----------------------------------------------------------------------------
fn select_units() -> Box<dyn Runnable> {
    SystemBuilder::new("mouse camera doda")
        .read_resource::<MouseButton>()
        .read_resource::<MousePos>()
        .read_resource::<Camera>()
        .write_resource::<SelectionBox>()
        .write_resource::<Drag>()
        .with_query(<Read<Pos>>::query().filter(tag::<Player>()))
        .build_thread_local(|cmd, world, resources, unit_positions| {
            let (mouse_btn, mouse_pos, camera, selection_box, drag) = resources;

            let mut pos = match camera.pos_from_camera(mouse_pos.global(), RAY_LENGTH) {
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

                    for (entity, unit_pos) in unit_positions.iter_entities(world) {
                        let unit_pos_2d = Vector2::new(unit_pos.0.x, unit_pos.0.z);
                        if selection.contains(unit_pos_2d.to_point()) {
                            cmd.add_tag(entity, Selected);
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


pub fn player_systems(builder: Builder) -> Builder {
    builder
        .add_thread_local(select_units())
}
