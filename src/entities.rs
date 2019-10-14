use classicube_sys::{Entities, EntityEvents, Event_RegisterInt, Event_UnregisterInt};
use std::{
  cell::RefCell,
  collections::HashMap,
  ffi::CStr,
  os::raw::{c_int, c_void},
  ptr,
};

thread_local! {
  pub static ENTITIES: RefCell<HashMap<usize, Entity>> = RefCell::new(HashMap::new());
}

#[derive(Debug, Clone)]
pub struct Entity {
  /// 255 is self entity
  pub id: usize,

  pub real_name: String,
  pub pos: [f32; 3],

  /// 0-360
  pub rot: [f32; 3],
}

fn get_entity(id: usize) -> Entity {
  let entity: &classicube_sys::Entity = unsafe { &*Entities.List[id] };

  let c_str = unsafe { CStr::from_ptr(&entity.DisplayNameRaw as *const i8) };
  let real_name = c_str.to_string_lossy().to_string();
  let pos = [entity.Position.X, entity.Position.Y, entity.Position.Z];
  let rot = [entity.RotX, entity.RotY, entity.RotZ];

  Entity {
    id,
    real_name,
    pos,
    rot,
  }
}

extern "C" fn on_entity_added(_obj: *mut c_void, id: c_int) {
  let id = id as usize;

  // print(format!("add ent {}", id));

  ENTITIES.with(|entities| {
    let mut entities = entities.borrow_mut();

    entities.insert(id, get_entity(id));
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
}
