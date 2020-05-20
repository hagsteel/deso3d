use gdnative::Vector2;
use legion::prelude::*;
use legion::systems::schedule::Builder;

use crate::input::{MouseButton, LMB};

const TILE_SIZE: f32 = 16.;

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


fn select_dnd_unit() -> Box<dyn Runnable> {
    SystemBuilder::new("select formation unit")
        .write_resource::<MouseButton>()
        // .read_resource::<FormationUI>()
        .with_query(<Read<FormationUnit>>::query())
        .with_query(<Read<FormationUnit>>::query())
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

            for (ent, unit) in units.iter_entities(world) {
                let mut rect = unit.0.get_rect();

                if rect.contains(mouse_pos.to_point()) {
                    cmd.add_tag(ent, FormationUnitSelected);
                }
            }
        })
}

fn deselect_unit() -> Box<dyn Runnable> {
    SystemBuilder::new("deselect formation unit")
        .read_resource::<MouseButton>()
        .read_resource::<FormationUI>()
        .with_query(<Write<FormationUnit>>::query().filter(tag::<FormationUnitSelected>()))
        .build_thread_local(|cmd, world, (mouse_btn, formation_ui), units| {
            if !mouse_btn.button_released(LMB) {
                return
            }

            for (ent, mut unit) in units.iter_entities_mut(world) {
                cmd.remove_tag::<FormationUnitSelected>(ent);

                // Snap here
                let mouse_pos = unsafe { formation_ui.0.get_local_mouse_position() };

                let snapped_pos = snap_to_pos(mouse_pos);
                unsafe { unit.0.set_position(snapped_pos, false) };
            }
        })
}

fn drag_unit() -> Box<dyn Runnable> {
    SystemBuilder::new("drag formation unit")
        .write_resource::<MouseButton>()
        .read_resource::<FormationUI>()
        .with_query(<Write<FormationUnit>>::query().filter(tag::<FormationUnitSelected>()))
        .build_thread_local(|cmd, world, (mouse_btn, formation_ui), units| unsafe {
            for mut unit in units.iter_mut(world) {
                let rect = unit.0.get_rect();
                let mouse_pos = formation_ui.0.get_local_mouse_position() - rect.size.to_vector() / 2.;
                unit.0.set_position(mouse_pos, false);
            }
        })
}

pub fn dnd_systems(builder: Builder) -> Builder {
    builder
        .add_thread_local(select_formation_unit())
        .add_thread_local(drag_formation_unit())
        .add_thread_local(deselect_formation_unit())
}

