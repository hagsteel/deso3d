use legion::prelude::*;
use legion::systems::schedule::Builder;

use crate::player::PlayerId;
use crate::movement::Pos;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Enemy;

pub struct DetectionRange(pub f32);

fn detect_player() -> Box<dyn Runnable> {
    SystemBuilder::new("detect player")
        .with_query(<(Read<Pos>)>::query().filter(tag::<PlayerId>()))
        .with_query(<(Read<DetectionRange>, Read<Pos>)>::query())
        .build_thread_local(|cmd, world, resources, (players, enemies)| {
            let player_positions = players.iter(world).map(|pos| pos.0).collect::<Vec<_>>();

            for (detection, pos) in enemies.iter(world) {
                for player_pos in &player_positions {
                    let dist = (*player_pos - pos.0).length();
                    if dist <= detection.0 {
                        // Player in range
                        eprintln!("{:?}", "player in range!");
                    }
                }
            }
        })
}


pub fn enemy_systems(builder: Builder) -> Builder {
    builder
        .add_thread_local(detect_player())
}
