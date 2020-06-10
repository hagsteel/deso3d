use gdnative::{Rect2, Vector2, Vector3};
use legion::prelude::*;
use legion::systems::schedule::Builder;
use serde::{Serialize, Deserialize};
use euclid::{Rotation2D, UnknownUnit};

use crate::camera::{Camera, Drag, SelectionBox, RAY_LENGTH};
use crate::input::{MouseButton, MousePos, LMB, RMB};
use crate::movement::{Destination, Pos, to_3d, to_2d};
use crate::formation::{FormationPos, index_to_x_y};

type Rotation2 = Rotation2D<f32, UnknownUnit, UnknownUnit>;

const OFFSET_MUL: f32 = 2.0;

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
        .with_query(<(Read<Pos>, Read<FormationPos>)>::query().filter(tag::<Selected>()))
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

            let positions = positions
                .iter_entities(world)
                .map(|(ent, (pos, formation_pos))| (ent, pos.0, formation_pos.0))
                .collect::<Vec<_>>();

            if positions.len() == 0 {
                return
            }

            let (offset, rotation) = {
                let mut offset_x = 0;
                let mut offset_y = usize::MAX;
                let mut dir = Vector2::zero();

                // Find the max x and the correct y
                for (_, pos, formation_pos) in &positions {
                    let (x, y) = index_to_x_y(*formation_pos as usize);

                    if x > offset_x {
                        offset_x = x;
                        offset_y = y;
                        dir = to_2d(dest_pos - *pos);
                    }
                    if y < offset_y {
                        offset_y = y;
                        dir = to_2d(dest_pos - *pos);
                    }
                }

                (
                    Vector2::new(offset_x as f32, offset_y as f32),
                    Rotation2::radians(dir.y.atan2(dir.x))
                )
            };

            for (ent, _, formation_pos) in &positions {
                let (x, y) = index_to_x_y(*formation_pos as usize);
                let formation_pos = (Vector2::new(x as f32, y as f32) - offset) * OFFSET_MUL;
                let formation_pos = rotation.transform_vector(formation_pos);
                let new_dest = dest_pos + to_3d(formation_pos);
                cmd.add_component(*ent, Destination(new_dest));
            }
        })
}


pub fn player_systems(builder: Builder) -> Builder {
    builder
        .add_thread_local(select_units())
        .add_thread_local(player_find_destinations())
}
