use bracket_lib::prelude::*;
use std::sync::Mutex;

lazy_static! {
    static ref RNG: Mutex<RandomNumberGenerator> = Mutex::new(RandomNumberGenerator::new());
}

pub fn reseed(new_seed: u64) {
    *RNG.lock().unwrap() = RandomNumberGenerator::seeded(new_seed);
}

pub fn roll_dice(n: i32, die_type: i32) -> i32 {
    RNG.lock().unwrap().roll_dice(n, die_type)
}

pub fn range(min: i32, max: i32) -> i32 {
    RNG.lock().unwrap().range(min, max)
}
