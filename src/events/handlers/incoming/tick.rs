use crate::{entity_emitter::ENTITY_EMITTERS, printer::PRINTER};

pub fn handle_tick() {
  PRINTER.lock().tick();

  let mut emitters = ENTITY_EMITTERS.lock();

  let mut to_remove = Vec::with_capacity(emitters.len());
  for (i, emitter) in emitters.iter_mut().enumerate() {
    if !emitter.update() {
      to_remove.push(i);
    }
  }

  // TODO can't you just use a for remove_id in ().rev()
  if !to_remove.is_empty() {
    for i in (0..emitters.len()).rev() {
      if to_remove.contains(&i) {
        emitters.remove(i);
      }
    }
  }
}
