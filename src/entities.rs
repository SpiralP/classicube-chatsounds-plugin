use classicube_sys::{Entities, EntityEvents, Event_RegisterInt, Event_UnregisterInt};
use std::{
  cell::RefCell,
  collections::HashMap,
  os::raw::{c_int, c_void},
  ptr,
};

/// 255 is self entity
pub const ENTITY_SELF_ID: usize = 255;

thread_local! {
  pub static ENTITIES: RefCell<HashMap<usize, Entity>> = RefCell::new(HashMap::new());
}

#[derive(Debug, Clone)]
pub struct Entity {
  pub id: usize,
}

impl Entity {
  fn get_entity(&self) -> &classicube_sys::Entity {
    unsafe { &*Entities.List[self.id] }
  }

  pub fn get_pos(&self) -> [f32; 3] {
    let entity = self.get_entity();
    [entity.Position.X, entity.Position.Y, entity.Position.Z]
  }

  /// 0-360
  pub fn get_rot(&self) -> [f32; 3] {
    let entity = self.get_entity();
    [entity.RotX, entity.RotY, entity.RotZ]
  }

  // pub fn get_real_name(&self) -> String {
  //   let entity = self.get_entity();
  //   let c_str = unsafe { CStr::from_ptr(&entity.DisplayNameRaw as *const i8) };
  //   c_str.to_string_lossy().to_string()
  // }
}

extern "C" fn on_entity_added(_obj: *mut c_void, id: c_int) {
  let id = id as usize;

  // print(format!("add ent {}", id));

  ENTITIES.with(|entities| {
    let mut entities = entities.borrow_mut();

    entities.insert(id, Entity { id });
  });
}

extern "C" fn on_entity_removed(_obj: *mut c_void, id: c_int) {
  let id = id as usize;

  // print(format!("removed ent {}", id));

  ENTITIES.with(|entities| {
    let mut entities = entities.borrow_mut();

    entities.remove(&id);
  });
}

pub fn load() {
  unsafe {
    Event_RegisterInt(
      &mut EntityEvents.Added,
      ptr::null_mut(),
      Some(on_entity_added),
    );
    Event_RegisterInt(
      &mut EntityEvents.Removed,
      ptr::null_mut(),
      Some(on_entity_removed),
    );
  }

  ENTITIES.with(|entities| {
    let mut entities = entities.borrow_mut();

    // add self which always exists?
    entities.insert(ENTITY_SELF_ID, Entity { id: ENTITY_SELF_ID });
  });
}

pub fn unload() {
  unsafe {
    Event_UnregisterInt(
      &mut EntityEvents.Added,
      ptr::null_mut(),
      Some(on_entity_added),
    );
    Event_UnregisterInt(
      &mut EntityEvents.Removed,
      ptr::null_mut(),
      Some(on_entity_removed),
    );
  }

  ENTITIES.with(|entities| {
    let mut entities = entities.borrow_mut();
    entities.clear();
  });
}
