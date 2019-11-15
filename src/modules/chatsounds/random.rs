use lazy_static::lazy_static;
use rand::prelude::*;
use rand_chacha::{rand_core::SeedableRng, ChaChaRng};
use std::{collections::HashMap, sync::Mutex};

lazy_static! {
  pub static ref ENTITY_COUNTS: Mutex<HashMap<u8, usize>> = Mutex::new(HashMap::new());
}

// TODO synced reset on new player/etc

pub fn get_rng(entity_id: u8) -> Box<dyn RngCore + Send> {
  let count = {
    let mut entity_counts = ENTITY_COUNTS.lock().unwrap();
    let count = entity_counts.entry(entity_id).or_insert(0);
    let n = *count as u64;
    *count += 1;
    n
  };

  let id = entity_id as u64;

  Box::new(ChaChaRng::seed_from_u64(256 * id + count))
}

#[test]
fn test_rand_index() {
  println!(
    "{:?}",
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9].choose(&mut get_rng(1)),
  );
  println!(
    "{:?}",
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9].choose(&mut get_rng(1)),
  );
  println!(
    "{:?}",
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9].choose(&mut get_rng(1)),
  );
  println!(
    "{:?}",
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9].choose(&mut get_rng(1)),
  );
  println!(
    "{:?}",
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9].choose(&mut get_rng(1)),
  );
}
