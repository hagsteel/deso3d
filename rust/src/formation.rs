use gdextras::node_ext::NodeExt;
use gdnative::{Color, Control, TextureRect, Vector2, Vector3};
use legion::prelude::*;
use legion::systems::schedule::Builder;

use crate::input::{MouseButton, LMB};

const TILE_SIZE: f32 = 16.;
const FORMATION_WIDTH: usize = 4;

// Formation related functions
fn index_to_x_y(index: usize) -> (usize, usize) {
    let y = index / FORMATION_WIDTH;
    let x = index - y * FORMATION_WIDTH;

    (x, y)
}

fn index_to_col(index: usize) -> usize {
    index / FORMATION_WIDTH
}

fn index_to_row(index: usize) -> usize {
    let col = index_to_col(index);
    let row = index - col * FORMATION_WIDTH;
    row
}

fn row_to_index(row: usize) -> Vec<usize> {
    let start = row * FORMATION_WIDTH;
    let end = row * FORMATION_WIDTH + FORMATION_WIDTH;
    (start..end).collect()
}

fn col_to_index(col: usize) -> Vec<usize> {
    (col..FORMATION_WIDTH * FORMATION_WIDTH)
        .step_by(FORMATION_WIDTH)
        .collect()
}

// *  Needs to work regardless of number of units
//
//    -----------------------
// *  Each unit has a formation position
// *  Each unit has a formation order
// *  Selected units are ordered by formation order

// TODO:
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

pub struct Formation {
    inner: Vec<Vector2>,
}

pub fn create_formation(dest: Vector3, unit_count: usize) -> Vec<Vector3> {
    let mut formation_positions = Vec::with_capacity(unit_count);

    let edge_size = (unit_count as f32).sqrt().ceil() as usize;

    for x in 0..edge_size {
        for z in 0..edge_size {
            let form_pos = dest + Vector3::new(x as f32, 0., z as f32) * 5.;
            formation_positions.push(form_pos);
        }
    }

    formation_positions
}

// -----------------------------------------------------------------------------
//     - Tags -
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FormationUnitSelected;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FormationUnitMoved;

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

                let mouse_pos = unsafe { formation_ui.0.get_local_mouse_position() };
                let mouse_pos = snap_to_pos(mouse_pos);

                // Check if there is already a unit at the final destination
                for mut unit in deselected_units.iter_mut(world) {
                    if unsafe { unit.0.get_rect() }.contains(mouse_pos.to_point()) {
                        unsafe { unit.0.set_position(start_pos, false) };
                    }
                }

                for (ent, (mut unit, _)) in units.iter_entities_mut(world) {
                    cmd.remove_tag::<FormationUnitSelected>(ent);
                    cmd.add_tag(ent, FormationUnitMoved);

                    unsafe {
                        unit.0.set_position(mouse_pos, false);

                        // Reparent the node to "Pending" again
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

// TODO: rewrite this system to be less clunky
// (review the multiple iterations over the grid)
fn done_moving() -> Box<dyn Runnable> {
    SystemBuilder::new("done moving")
        .with_query(<(
            Write<FormationPos>,
            Write<FormationOrder>,
            Read<FormationUnit>,
        )>::query())
        .with_query(<Read<FormationUnit>>::query().filter(tag::<FormationUnitMoved>()))
        .build_thread_local(|cmd, world, resources, (units, done_moving_unit)| {
            let entities = done_moving_unit
                .iter_entities_mut(world)
                .map(|(ent, _)| ent)
                .collect::<Vec<_>>();

            if entities.len() == 0 {
                return;
            }

            for ent in &entities {
                cmd.remove_tag::<FormationUnitMoved>(*ent);
            }

            let mut grid = Vec::with_capacity(16);
            let mut x_range = (0..4).collect::<Vec<_>>();
            x_range.reverse();

            // Create grid (the entire grid, 4x4)
            for x in x_range {
                for y in 0..4 {
                    grid.push((Vector2::new(x as f32, y as f32), false));
                }
            }

            for (mut pos, mut order, unit) in units.iter_mut(world) {
                // Find the unit on the grid.
                // Remove all empty columns and rows
                let unit_pos = unsafe { unit.0.get_position() } / 16.;

                for (grid_pos, occupied) in &mut grid {
                    if *grid_pos == unit_pos {
                        *occupied = true;
                    }
                }
            }

            // Filter out rows / columns that have no unit in them

            let mut keep_rows = Vec::new();
            let mut keep_cols = Vec::new();

            for (index, (pos, occupied)) in grid.iter().enumerate() {
                if *occupied {
                    let row = index_to_row(index);
                    let col = index_to_col(index);

                    // if !keep_rows.contains(&row) {
                    //     keep_rows.push(row);
                    // }

                    if !keep_cols.contains(&col) {
                        keep_cols.push(col);
                    }
                }
            }

            let mut keep = Vec::new();
            for row in &keep_rows {
                let mut indices = row_to_index(*row);
                keep.append(&mut indices);
            }

            for col in &keep_cols {
                let mut indices = col_to_index(*col);
                keep.append(&mut indices);
            }

            keep.sort();
            keep.dedup();
            keep.reverse();

            eprintln!("{:?}", keep);
            keep.into_iter().for_each(|index| {
                grid.remove(index);
            });

            for (pos, _) in &grid {
                eprintln!("{}", pos);
            }
        })
}

pub fn formation_systems(builder: Builder) -> Builder {
    builder
        .add_thread_local(select_formation_unit())
        .add_thread_local(drag_formation_unit())
        .add_thread_local(deselect_formation_unit())
        .add_thread_local(done_moving())
}

#[cfg(test)]
mod test {
    use super::*;
    use gdnative::Vector2;

    // #[test]
    // fn test_square_formation() {
    //     let dest = Vector3::new(0., 0., 0.);
    //     let unit_positions = vec![
    //         Vector3::new(-10., 0., 10.),
    //         Vector3::new(10., 0., 10.),
    //         Vector3::new(-10., 0., -10.),
    //     ];

    //     let formation = create_formation(dest, FormationShape::Square, unit_positions);

    //     assert!(formation.contains(&dest));
    //     let formation = formation
    //         .into_iter()
    //         .map(|pos| Vector2::new(pos.x, pos.z))
    //         .collect::<Vec<_>>();
    //     eprintln!("{:#?}", formation);
    // }

    #[test]
    fn test_col_to_index() {
        let grid = (0..4 * 4).collect::<Vec<_>>();
        let first_col = 0;
        let second_col = 1;
        let third_col = 2;
        let fourth_col = 3;

        assert_eq!(col_to_index(first_col), vec![0, 4, 8, 12]);
        assert_eq!(col_to_index(second_col), vec![1, 5, 9, 13]);
        assert_eq!(col_to_index(third_col), vec![2, 6, 10, 14]);
        assert_eq!(col_to_index(fourth_col), vec![3, 7, 11, 15]);
    }

    #[test]
    fn test_row_to_index() {
        let grid = (0..4 * 4).collect::<Vec<_>>();
        let first_row = 0;
        let second_row = 1;

        assert_eq!(row_to_index(first_row), vec![0, 1, 2, 3]);
        assert_eq!(row_to_index(second_row), vec![4, 5, 6, 7]);
    }
}
