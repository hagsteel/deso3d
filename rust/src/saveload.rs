use std::env::current_dir;
use std::fs::File;
use std::io::Result;
use std::path::PathBuf;

use legion::prelude::*;
use serde::{Deserialize, Serialize};

// use crate::combat::{AttackCooldown, AttackRange, AttackResponse};
use crate::gameworld::with_world;
use crate::player::PlayerId;
// use crate::unit::{Hitpoints, UnitPos, Speed};
use crate::movement::{Speed, Pos};
// use crate::enemy::Enemy;

fn file_path(slot: u8) -> Result<PathBuf> {
    let mut path = current_dir()?;
    path.push(format!("save_{}.json", slot));
    Ok(path)
}

type PlayerUnitData = (
    PlayerId,
    Pos,
    // Hitpoints,
    // AttackRange,
    // AttackCooldown,
    // AttackResponse,
    Speed,
);

type PlayerUnitDataQuery = (
    Read<PlayerId>,
    Read<Pos>,
    // Read<Hitpoints>,
    // Read<AttackRange>,
    // Read<AttackCooldown>,
    // Read<AttackResponse>,
    Read<Speed>,
);

// type EnemyUnitData = (
//     Enemy,
//     UnitPos,
//     Hitpoints,
//     AttackRange,
//     AttackCooldown,
//     AttackResponse,
//     Speed,
// );

// type EnemyUnitDataQuery = (
//     Read<Enemy>,
//     Read<UnitPos>,
//     Read<Hitpoints>,
//     Read<AttackRange>,
//     Read<AttackCooldown>,
//     Read<AttackResponse>,
//     Read<Speed>,
// );

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveData {
    pub player_units: Vec<PlayerUnitData>,
    // pub enemy_units: Vec<EnemyUnitData>,
}

impl SaveData {
    pub fn new() -> Self {
        Self {
            player_units: Vec::with_capacity(4),
            // enemy_units: Vec::new(),
        }
    }
}

pub fn save(slot: u8) -> Result<()> {
    let mut file = match File::create(file_path(slot)?) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("{:?}", e);
            return Err(e);
        }
    };

    let mut save_data = SaveData::new();

    with_world(|world| {
        for (player_id, pos, /*hp, attack_range, attack_cooldown, attack_response,*/ speed) in
            PlayerUnitDataQuery::query().iter(world)
        {
            save_data.player_units.push((
                *player_id,
                *pos,
                // *hp,
                // *attack_range,
                // *attack_cooldown,
                // *attack_response,
                *speed,
            ));
        }

        // for (enemy_id, unit_pos, hp, attack_range, attack_cooldown, attack_response, speed) in
        //     EnemyUnitDataQuery::query().iter(world)
        // {
        //     save_data.enemy_units.push((
        //         *enemy_id,
        //         *unit_pos,
        //         *hp,
        //         *attack_range,
        //         *attack_cooldown,
        //         *attack_response,
        //         *speed,
        //     ));
        // }
    });

    serde_json::to_writer_pretty(&mut file, &save_data)?;

    Ok(())
}

pub fn load(slot: u8) -> Result<SaveData> {
    let file = File::open(file_path(slot)?)?;

    match serde_json::from_reader(&file) {
        Ok(save_data) => Ok(save_data),
        Err(_) => panic!("Could not deserialize game state"),
    }
}
