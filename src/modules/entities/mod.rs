mod entity;

pub use self::entity::{Entity, ENTITY_SELF_ID};
use crate::modules::Module;
use classicube_sys::{EntityEvents, Event_RegisterInt, Event_UnregisterInt};
use std::{
  collections::HashMap,
  os::raw::{c_int, c_void},
};

pub struct EntitiesModule {
  entities: Box<HashMap<usize, Entity>>,
}

impl EntitiesModule {
  pub fn new() -> Self {
    Self {
      entities: Box::new(HashMap::new()),
    }
  }

  pub fn get(&self, id: usize) -> Option<&Entity> {
    self.entities.get(&id)
  }
}

impl Module for EntitiesModule {
  fn load(&mut self) {
    let ptr: *mut EntitiesModule = self;

    unsafe {
      Event_RegisterInt(
        &mut EntityEvents.Added,
        ptr as *mut c_void,
        Some(on_entity_added),
      );

      Event_RegisterInt(
        &mut EntityEvents.Removed,
        ptr as *mut c_void,
        Some(on_entity_removed),
      );
    }

    // add self which always exists?
    self
      .entities
      .insert(ENTITY_SELF_ID, Entity { id: ENTITY_SELF_ID });
  }

  fn unload(&mut self) {
    let ptr: *mut EntitiesModule = self;

    unsafe {
      Event_UnregisterInt(
        &mut EntityEvents.Added,
        ptr as *mut c_void,
        Some(on_entity_added),
      );

      Event_UnregisterInt(
        &mut EntityEvents.Removed,
        ptr as *mut c_void,
        Some(on_entity_removed),
      );
    }
  }
}

extern "C" fn on_entity_added(obj: *mut c_void, id: c_int) {
  let module = obj as *mut EntitiesModule;
  let module = unsafe { &mut *module };
  let id = id as usize;

  // print(format!("add ent {}", id));

  module.entities.insert(id, Entity { id });
}

extern "C" fn on_entity_removed(obj: *mut c_void, id: c_int) {
  let module = obj as *mut EntitiesModule;
  let module = unsafe { &mut *module };
  let id = id as usize;

  // print(format!("removed ent {}", id));

  module.entities.remove(&id);
}
