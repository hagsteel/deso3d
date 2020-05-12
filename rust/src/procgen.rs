use gdnative::Vector2;
use std::hash::Hasher;
use std::u32;
use twox_hash::XxHash;

use crate::tilemap::Coords;


pub fn pack_vec2(pos: Vector2) -> u64 {
    pack(pos.x as i32, pos.y as i32)
}

fn pack(x: i32, y: i32) -> u64 {
    let x = x as i64;
    let y = y as i64;

    ((x << 32) | (y & u32::MAX as i64)) as u64
}

pub fn random_choice<T>(slice: &[T], seed: u64) -> &T {
    let hasher = XxHash::with_seed(seed);
    let hash = hasher.finish();
    let index = hash as usize % slice.len();

    &slice[index]
}

pub fn random_bool(seed: u64, weight: usize) -> bool {
    let hasher = XxHash::with_seed(seed);
    let hash = hasher.finish();
    let val = hash as usize % 10000;
    val <= weight
}
