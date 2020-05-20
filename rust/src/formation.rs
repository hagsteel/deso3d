use gdnative::{Color, TextureRect, Vector2, Vector3};
use legion::prelude::*;
use legion::systems::schedule::Builder;

use crate::player::Selected;
use crate::input::{MouseButton, LMB};

// *  Needs to work regardless of number of units
//
//    -----------------------
// *  Each unit has a formation position
// *  Each unit has a formation order
// *  Selected units are ordered by formation order

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

#[derive(Debug)]
pub struct FormationOrder(u8);

// -----------------------------------------------------------------------------
//     - Systems -
// -----------------------------------------------------------------------------
fn select_formation_unit() -> Box<dyn Runnable> {
    SystemBuilder::new("select formation unit")
        .write_resource::<MouseButton>()
        .read_resource::<FormationUI>()
        .with_query(<Read<FormationUnit>>::query())
        .build_thread_local(|cmd, world, (mouse_btn, formation_ui), query| {
            if !mouse_btn.button_pressed(LMB) {
                return;
            }

            let mouse_pos = unsafe { formation_ui.0.get_global_mouse_position() };

            // if the mouse position is within the area of the 
            // formation ui then consume the mouse input

            let mut rect = unsafe { formation_ui.0.get_rect() };
            let scale = unsafe { formation_ui.0.get_scale() };
            rect.size.width *= scale.x;
            rect.size.height *= scale.y;
            if !rect.contains(mouse_pos.to_point()) {
                return
            }
            eprintln!("{} | {:#?}", mouse_pos, rect);

            mouse_btn.consume();
        })
}

fn drag_formation_unit() -> Box<dyn Runnable> {
    SystemBuilder::new("drag formation unit")
        .build_thread_local(|cmd, world, resources, query| {
            
        })
}

pub fn formation_systems(builder: Builder) -> Builder {
    builder
        .add_thread_local(select_formation_unit())
        .add_thread_local(drag_formation_unit())
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
            // Vector3::new(10., 0., -10.),
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
