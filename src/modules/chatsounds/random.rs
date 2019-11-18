use lazy_static::lazy_static;
use rand::prelude::*;
use rand_chacha::{rand_core::SeedableRng, ChaChaRng};
use std::{
  collections::{hash_map::DefaultHasher, HashMap},
  hash::{Hash, Hasher},
  sync::Mutex,
};

// TODO make this a field on ChatsoundsModule/EventListener
lazy_static! {
  pub static ref ENTITY_COUNTS: Mutex<HashMap<String, usize>> = Mutex::new(HashMap::new());
}

// TODO synced reset on new player/etc
// TODO self id is 255 but others don't see 255!!

pub fn sync_reset() {
  let mut entity_counts = ENTITY_COUNTS.lock().unwrap();
  entity_counts.clear();
}

pub fn update_chat_count<S: AsRef<str>>(real_name: S) {
  let real_name = real_name.as_ref();

  let mut entity_counts = ENTITY_COUNTS.lock().unwrap();
  let count = entity_counts.entry(real_name.to_string()).or_insert(0);
  *count += 1;
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
  let mut s = DefaultHasher::new();
  t.hash(&mut s);
  s.finish()
}

#[derive(PartialEq, Eq, Hash)]
struct HashedInfo<'a> {
  real_name: &'a str,
  messages_said: usize,
}

pub fn get_rng<S: AsRef<str>>(real_name: S) -> Box<dyn RngCore + Send> {
  let real_name = real_name.as_ref();

  // id isn't synced between players (since self is 255)
  // so we use real_name as the unique, shared field

  let mut entity_counts = ENTITY_COUNTS.lock().unwrap();
  let messages_said = *entity_counts.entry(real_name.to_string()).or_insert(0);

  let hash = calculate_hash(&HashedInfo {
    real_name,
    messages_said,
  });

  Box::new(ChaChaRng::seed_from_u64(hash))
}
