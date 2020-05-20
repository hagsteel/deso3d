use gdextras::node_ext::NodeExt;
use gdnative::{Color, Control, TextureRect, Vector2, Vector3};
use legion::prelude::*;
use legion::systems::schedule::Builder;

use crate::input::{MouseButton, LMB};

const TILE_SIZE: f32 = 16.;

// *  Needs to work regardless of number of units
//
//    -----------------------
// *  Each unit has a formation position
// *  Each unit has a formation order
// *  Selected units are ordered by formation order

// TODO
// [ ] Snapping
// [ ] Updating the FormationPosition

fn pos_to_coords(pos: Vector2) -> Vector2 {
    (pos / TILE_SIZE).floor()
}

fn coords_to_pos(coords: Vector2) -> Vector2 {
    coords * TILE_SIZE
}

fn snap_to_pos(pos: Vector2) -> Vector2 {
    let coords = pos_to_coords(pos);
    coords_to_pos(coords)
}

pub enum FormationShape {
    Line,
    Square,
}

pub struct Formation {
    inner: Vec<Vector2>,
}

pub fn create_formation(
    dest: Vector3,
    formation: FormationShape,
    unit_count: usize,
) -> Vec<Vector3> {
    let mut formation_positions = Vec::with_capacity(unit_count);

    use FormationShape::*;
    match formation {
        Line => {}
        Square => {
            let edge_size = (unit_count as f32).sqrt().ceil() as usize;

            for x in 0..edge_size {
                for z in 0..edge_size {
                    let form_pos = dest + Vector3::new(x as f32, 0., z as f32) * 5.;
                    formation_positions.push(form_pos);
                }
            }
        }
    }

    formation_positions
}

// -----------------------------------------------------------------------------
//     - Tags -
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FormationUnitSelected;

// -----------------------------------------------------------------------------
//     - Resources -
// -----------------------------------------------------------------------------
pub struct FormationUI(TextureRect);

impl FormationUI {
    pub fn new(inner: TextureRect) -> Self {
        Self(inner)
    }
}

unsafe impl Send for FormationUI {}
unsafe impl Sync for FormationUI {}

pub struct FormationUnit(TextureRect);

impl FormationUnit {
    pub fn new(inner: TextureRect) -> Self {
        Self(inner)
    }

    pub fn set_color(&mut self, color: Color) {
        unsafe { self.0.set_modulate(color) };
    }
}

unsafe impl Send for FormationUnit {}
unsafe impl Sync for FormationUnit {}

// -----------------------------------------------------------------------------
//     - Components -
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub struct FormationPos(Vector2);

impl FormationPos {
    pub fn new(pos: Vector2) -> Self {
        Self(pos)
    }
}

#[derive(Debug)]
pub struct FormationOrder(u8);

impl FormationOrder {
    pub fn new(index: u8) -> Self {
        Self(index)
    }
}

#[derive(Debug, Clone, Copy)]
struct StartDrag(Vector2);

// -----------------------------------------------------------------------------
//     - Systems -
// -----------------------------------------------------------------------------
fn select_formation_unit() -> Box<dyn Runnable> {
    SystemBuilder::new("select formation unit")
        .write_resource::<MouseButton>()
        .read_resource::<FormationUI>()
        .with_query(<Write<FormationUnit>>::query())
        .build_thread_local(|cmd, world, (mouse_btn, formation_ui), units| unsafe {
            if !mouse_btn.button_pressed(LMB) {
                return;
            }

            let mouse_pos = formation_ui.0.get_local_mouse_position();

            let mut rect = formation_ui.0.get_rect();
            rect.origin = Vector2::zero().to_point();
            if !rect.contains(mouse_pos.to_point()) {
                return;
            }
            mouse_btn.consume();

            for (ent, mut unit) in units.iter_entities_mut(world) {
                let rect = unit.0.get_rect();

                if rect.contains(mouse_pos.to_point()) {
                    cmd.add_component(ent, StartDrag(mouse_pos));
                    cmd.add_tag(ent, FormationUnitSelected);

                    let mut pending = formation_ui.0.get_and_cast::<Control>("Pending").unwrap();
                    let mut moving = formation_ui.0.get_and_cast::<Control>("Moving").unwrap();
                    pending.remove_child(Some(unit.0.to_node()));
                    moving.add_child(Some(unit.0.to_node()), false);
                    unit.0.set_owner(Some(moving.to_node()));
                }
            }
        })
}

fn deselect_formation_unit() -> Box<dyn Runnable> {
    SystemBuilder::new("deselect formation unit")
        .read_resource::<MouseButton>()
        .read_resource::<FormationUI>()
        .with_query(<Write<FormationUnit>>::query())
        .with_query(
            <(Write<FormationUnit>, Read<StartDrag>)>::query()
                .filter(tag::<FormationUnitSelected>()),
        )
        .build_thread_local(
            |cmd, world, (mouse_btn, formation_ui), (deselected_units, units)| {
                if !mouse_btn.button_released(LMB) {
                    return;
                }

                let start_pos = match units.iter_mut(world).next() {
                    Some((_, s)) => snap_to_pos(s.0),
                    None => return,
                };

                // Snap here
                let mouse_pos = unsafe { formation_ui.0.get_local_mouse_position() };

                for mut unit in deselected_units.iter_mut(world) {
                    if unsafe { unit.0.get_rect() }.contains(mouse_pos.to_point()) {
                        unsafe { unit.0.set_position(start_pos, false) };
                    }
                }

                for (ent, (mut unit, _)) in units.iter_entities_mut(world) {
                    cmd.remove_tag::<FormationUnitSelected>(ent);

                    let snapped_pos = snap_to_pos(mouse_pos);
                    unsafe {
                        unit.0.set_position(snapped_pos, false);
                        let mut pending =
                            formation_ui.0.get_and_cast::<Control>("Pending").unwrap();
                        let mut moving = formation_ui.0.get_and_cast::<Control>("Moving").unwrap();
                        moving.remove_child(Some(unit.0.to_node()));
                        pending.add_child(Some(unit.0.to_node()), false);
                        unit.0.set_owner(Some(pending.to_node()));
                    }
                }
            },
        )
}

fn drag_formation_unit() -> Box<dyn Runnable> {
    SystemBuilder::new("drag formation unit")
        .read_resource::<FormationUI>()
        .with_query(<Write<FormationUnit>>::query().filter(tag::<FormationUnitSelected>()))
        .build_thread_local(|_, world, formation_ui, units| unsafe {
            for mut unit in units.iter_mut(world) {
                let rect = unit.0.get_rect();
                let mouse_pos =
                    formation_ui.0.get_local_mouse_position() - rect.size.to_vector() / 2.;
                unit.0.set_position(mouse_pos, false);
            }
        })
}

pub fn formation_systems(builder: Builder) -> Builder {
    builder
        .add_thread_local(select_formation_unit())
        .add_thread_local(drag_formation_unit())
        .add_thread_local(deselect_formation_unit())
}

#[cfg(test)]
mod test {
    use super::*;
    use gdnative::Vector2;

    #[test]
    fn test_square_formation() {
        let dest = Vector3::new(0., 0., 0.);
        let unit_positions = vec![
            Vector3::new(-10., 0., 10.),
            Vector3::new(10., 0., 10.),
            Vector3::new(-10., 0., -10.),
        ];

        let formation = create_formation(dest, FormationShape::Square, unit_positions);

        assert!(formation.contains(&dest));
        let formation = formation
            .into_iter()
            .map(|pos| Vector2::new(pos.x, pos.z))
            .collect::<Vec<_>>();
        eprintln!("{:#?}", formation);
    }
}
