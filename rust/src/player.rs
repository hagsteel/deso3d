use gdnative::{Rect2, Vector2, Vector3};
use legion::prelude::*;
use legion::systems::schedule::Builder;
use serde::{Serialize, Deserialize};

use crate::camera::{Camera, Drag, SelectionBox, RAY_LENGTH};
use crate::input::{MouseButton, MousePos, LMB, RMB};
use crate::movement::{Destination, Pos};
use crate::unit::Unit;

const OFFSET_MUL: f32 = 5.0;

// -----------------------------------------------------------------------------
//     - Tags -
// -----------------------------------------------------------------------------
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Selected;

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub struct PlayerId(u8);

impl PlayerId {
    pub fn new(id: u8) -> Self {
        Self(id)
    }
}

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
        .with_query(<Read<Pos>>::query().filter(tag::<PlayerId>()))
        .build_thread_local(|cmd, world, resources, unit_positions| {
            let (mouse_btn, mouse_pos, camera, selection_box, drag) = resources;

            let mut pos = match camera.pos_from_camera(mouse_pos.global(), RAY_LENGTH, 2) {
                Some(p) => p,
                None => return,
            };

            if !mouse_btn.button_pressed(LMB) {
                if let Drag::Start(start_pos) = drag as &mut Drag {
                    // Selection
                    let start_2d = Vector2::new(start_pos.x, start_pos.z).to_point();
                    let end_2d = Vector2::new(pos.x, pos.z).to_point();
                    let size = (start_2d - end_2d).abs();
                    let point = Vector2::new(start_2d.x.min(end_2d.x), start_2d.y.min(end_2d.y));
                    let selection = Rect2::new(point.to_point(), size.to_size());

                    for (entity, _) in unit_positions.iter_entities(world) {
                        cmd.remove_tag::<Selected>(entity);
                    }

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

fn player_find_destinations() -> Box<dyn Runnable> {
    SystemBuilder::new("player find destination")
        .read_resource::<Camera>()
        .write_resource::<MouseButton>()
        .read_resource::<MousePos>()
        .with_query(<Read<Pos>>::query().filter(tag::<Selected>()))
        .build_thread_local(|cmd, world, resources, positions| {
            let (camera, mouse_btn, mouse_pos) = resources;

            if !mouse_btn.button_pressed(RMB) {
                return;
            }

            mouse_btn.consume();

            let dest_pos = match camera.pos_from_camera(mouse_pos.global(), RAY_LENGTH, 1) {
                Some(p) => p,
                None => return,
            };

            let mut positions_sorted = positions.iter_entities(world).map(|(ent, pos)| (ent, pos.0)).collect::<Vec<_>>();
            positions_sorted.sort_by(|a, b| {
                let a_len = (a.1 - dest_pos).length();
                let b_len = (b.1 - dest_pos).length();
                b_len.partial_cmp(&a_len).unwrap()
            });

            let mut prev_dest = dest_pos;
            while let Some((ent, pos)) = positions_sorted.pop() {
                cmd.add_component(ent, Destination(prev_dest));
                let direction = (dest_pos - pos).normalize();
                let offset = direction * OFFSET_MUL;
                prev_dest -= offset;
            }
        })
}


pub fn player_systems(builder: Builder) -> Builder {
    builder
        .add_thread_local(select_units())
        .add_thread_local(player_find_destinations())
}
