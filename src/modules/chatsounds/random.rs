use lazy_static::lazy_static;
use rand::prelude::*;
use rand_chacha::{rand_core::SeedableRng, ChaChaRng};
use std::{collections::HashMap, sync::Mutex};

lazy_static! {
  pub static ref ENTITY_COUNTS: Mutex<HashMap<usize, usize>> = Mutex::new(HashMap::new());
}

// TODO synced reset on new player/etc

pub fn rand_index<T>(vec: &[T], entity_id: usize) -> Option<&T> {
  let count = {
    let mut entity_counts = ENTITY_COUNTS.lock().unwrap();
    let count = entity_counts.entry(entity_id).or_insert(0);
    let n = *count as u64;
    *count += 1;
    n
  };

  let id = entity_id as u64;

  let mut rng = ChaChaRng::seed_from_u64(256 * id + count);

  if vec.is_empty() {
    None
  } else {
    let index: usize = rng.gen_range(0, vec.len());
    Some(&vec[index])
  }
}

#[test]
fn test_rand_index() {
  println!("{:?}", rand_index(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9], 1));
  println!("{:?}", rand_index(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9], 1));
  println!("{:?}", rand_index(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9], 1));
  println!("{:?}", rand_index(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9], 1));
  println!("{:?}", rand_index(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9], 1));
}
