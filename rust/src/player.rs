use euclid::{Rotation2D, UnknownUnit};
use gdnative::{Rect2, Vector2, Vector3, Ptr};
use legion::prelude::*;
use legion::systems::schedule::Builder;
use serde::{Deserialize, Serialize};

use crate::camera::{Camera, Drag, SelectionBox, RAY_LENGTH};
use crate::contextmenu::ContextMenuNode;
use crate::formation::{index_to_x_y, FormationPos};
use crate::input::{MouseButton, MousePos, LMB, RMB};
use crate::movement::{to_2d, to_3d, Destination, Pos};
use crate::unit::Unit;
use crate::gameworld::ClickedState;
use crate::safe;

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
            let selection_box = unsafe { selection_box.0.assume_safe() };

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

                unsafe { selection_box.set_scale(Vector3::zero()) };
                drag.clear();
                return;
            }

            pos.y = 1.0;

            match drag as &mut Drag {
                Drag::Empty => {
                    drag.set_start(pos);
                    unsafe { selection_box.set_translation(pos) };
                }
                Drag::Start(start_pos) => {
                    let mut size = pos - *start_pos;
                    size.y = 0.3;
                    selection_box.set_scale(size);
                    let translation = pos - size / 2.;
                    selection_box.set_translation(translation);
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

            let dest_pos = match camera.pos_from_camera(mouse_pos.global(), RAY_LENGTH, 2) {
                Some(p) => p,
                None => return,
            };

            let positions = positions
                .iter_entities(world)
                .map(|(ent, (pos, formation_pos))| (ent, pos.0, formation_pos.0))
                .collect::<Vec<_>>();

            if positions.len() == 0 {
                return;
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
                    Rotation2::radians(dir.y.atan2(dir.x)),
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

fn player_open_context_menu() -> Box<dyn Runnable> {
    SystemBuilder::new("player open context menu")
        .read_resource::<Camera>()
        .write_resource::<MouseButton>()
        .read_resource::<MousePos>()
        .with_query(<(Read<Unit>, Write<ContextMenuNode>)>::query().filter(tag::<PlayerId>()))
        .build_thread_local(|cmd, world, resources, units| {
            let (camera, mouse_btn, mouse_pos) = resources;

            if !mouse_btn.button_pressed(RMB) {
                return;
            }

            // If the unit is clicked then consume the input
            let dict = camera.object_from_camera(mouse_pos.global(), RAY_LENGTH, 4);
            if dict.is_empty() {
                units
                    .iter_mut(world)
                    .for_each(|(_, mut menu)| unsafe { menu.0.assume_safe().set_visible(false) });
                return;
            }

            let collider_id = match dict.get(&"collider_id".into()).try_to_i64() {
                Some(r) => r,
                None => return,
            };

            for (unit, menu) in units.iter_mut(world) {
                let unit = unsafe { unit.inner.assume_safe() };
                let menu = unsafe { menu.0.assume_safe() };

                let instance_id = unit.get_instance_id();
                if instance_id == collider_id {
                    menu.set_position(mouse_pos.global(), false);
                    menu.set_visible(true);
                } else {
                    menu.set_visible(false);
                }
            }

            // 1.  Hide all context menues
            // 2.  Show context menu
        })
}

fn something_clicked() -> Box<dyn Runnable> {
    SystemBuilder::new("something_clicked")
        .write_resource::<ClickedState>()
        .with_query(<Write<ContextMenuNode>>::query())
        .build_thread_local(|cmd, world, state, query| {

            if state.clicked {
                eprintln!("{:?}", "it was clicked!");
                state.clicked = false;

                for mut menu in query.iter_mut(world) {
                    let menu = unsafe { menu.0.assume_safe() };
                    menu.set_visible(false);
                }
            }
            
        })
}

pub fn player_systems(builder: Builder) -> Builder {
    builder
        .add_thread_local(select_units())
        .add_thread_local(player_open_context_menu())
        .add_thread_local(player_find_destinations())
        .add_thread_local(something_clicked())
}
