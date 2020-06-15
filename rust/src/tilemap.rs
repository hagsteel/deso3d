use gdnative::{GridMap, Vector2};
use legion::prelude::*;

use crate::procgen::{pack_vec2, random_bool};

pub struct TileMap(pub GridMap);

unsafe impl Send for TileMap {}
unsafe impl Sync for TileMap {}

// TODO: delete this (why?)
pub struct Coords {
    cells: Vec<Vector2>,
}

impl Coords {
    pub fn new() -> Self {
        let cells = make_cells();
        Self { cells }
    }
}

fn make_cells() -> Vec<Vector2> {
    let mut v = Vec::with_capacity(100 * 100);
    for x in -1..100 {
        for z in -1..100 {
            v.push(Vector2::new(x as f32, z as f32))
        }
    }

    v
}

pub fn draw_tilemap() -> Box<dyn Runnable> {
    SystemBuilder::new("draw tilemap")
        .write_resource::<Coords>()
        .write_resource::<TileMap>()
        .build_thread_local(|_, _, (coords, tilemap), _| {
            if coords.cells.len() == 0 {
                return;
            }

            for cell in coords.cells.drain(..) {
                let x = cell.x as i64;
                let y = 0;
                let z = cell.y as i64;

                let seed = pack_vec2(cell);
                let cell_type = if random_bool(seed, 100) { 1 } else { 0 };

                // random_choice(&[2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1], seed);
                unsafe {
                    tilemap.0.set_cell_item(x, y, z, cell_type, 0);
                }
            }
        })
}
