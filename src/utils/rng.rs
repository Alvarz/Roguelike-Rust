use bracket_lib::prelude::*;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

lazy_static! {
    static ref CURRENT_SEED: Mutex<u64> = Mutex::new(generate_random_seed());
    static ref RNG: Mutex<RandomNumberGenerator> =
        Mutex::new(RandomNumberGenerator::seeded(*CURRENT_SEED.lock().unwrap()));
}
fn generate_random_seed() -> u64 {
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    bracket_lib::prelude::console::log(format!("seed: {}", seed));
    seed
}

// Function to get the current seed value
pub fn get_current_seed() -> u64 {
    *CURRENT_SEED.lock().unwrap()
}

pub fn reseed(new_seed: u64) {
    *CURRENT_SEED.lock().unwrap() = new_seed; // Update the current seed
    *RNG.lock().unwrap() = RandomNumberGenerator::seeded(new_seed);
}

pub fn roll_dice(n: i32, die_type: i32) -> i32 {
    RNG.lock().unwrap().roll_dice(n, die_type)
}

pub fn range(min: i32, max: i32) -> i32 {
    RNG.lock().unwrap().range(min, max)
}
