mod entry;

pub use self::entry::TabListEntry;
use crate::modules::Module;
use classicube_sys::{Event_RegisterInt, Event_UnregisterInt, TabListEvents};
use std::{
  collections::HashMap,
  os::raw::{c_int, c_void},
};

pub struct TabListModule {
  entries: HashMap<usize, TabListEntry>,
}

impl TabListModule {
  pub fn new() -> Self {
    Self {
      entries: HashMap::new(),
    }
  }

  pub fn find_entity_id_by_name(&self, full_nick: String) -> Option<usize> {
    // try exact match
    self
      .entries
      .iter()
      .find_map(|(id, entry)| {
        if entry.nick_name == full_nick {
          Some(*id)
        } else {
          None
        }
      })
      .or_else(|| {
        // match from the right, choose the one with most chars matched
        let mut id_positions: Vec<(usize, usize)> = self
          .entries
          .iter()
          .filter_map(|(id, entry)| {
            // full_nick &g[&x&7___&g] &m___&0 Cjnator38
            // real_nick &g&m___&0 Cjnator38

            // &7<Map>&r&r[&f/dl&r] Empy: &fthis milk chocolate fuck has

            // remove color at beginning
            let full_nick = if full_nick.len() >= 2 && full_nick.starts_with('&') {
              let (_color, full_nick) = full_nick.split_at(2);
              full_nick
            } else {
              full_nick.as_str()
            };
            let real_nick = if entry.nick_name.len() >= 2 && entry.nick_name.starts_with('&') {
              let (_color, real_nick) = entry.nick_name.split_at(2);
              real_nick
            } else {
              entry.nick_name.as_str()
            };

            full_nick.rfind(&real_nick).map(|pos| (*id, pos))
          })
          .collect();

        // choose smallest position, or "most chars matched"
        id_positions.sort_unstable_by(|(id1, pos1), (id2, pos2)| {
          pos1
            .partial_cmp(pos2)
            .unwrap()
            .then_with(|| id1.partial_cmp(&id2).unwrap())
        });

        id_positions.first().map(|(id, _pos)| *id)
      })
  }
}

impl Module for TabListModule {
  fn load(&mut self) {
    let ptr: *mut TabListModule = self;

    unsafe {
      Event_RegisterInt(
        &mut TabListEvents.Added,
        ptr as *mut c_void,
        Some(on_tablist_added),
      );
      Event_RegisterInt(
        &mut TabListEvents.Changed,
        ptr as *mut c_void,
        Some(on_tablist_changed),
      );
      Event_RegisterInt(
        &mut TabListEvents.Removed,
        ptr as *mut c_void,
        Some(on_tablist_removed),
      );
    }
  }

  fn unload(&mut self) {
    let ptr: *mut TabListModule = self;

    unsafe {
      Event_UnregisterInt(
        &mut TabListEvents.Added,
        ptr as *mut c_void,
        Some(on_tablist_added),
      );
      Event_UnregisterInt(
        &mut TabListEvents.Changed,
        ptr as *mut c_void,
        Some(on_tablist_changed),
      );
      Event_UnregisterInt(
        &mut TabListEvents.Removed,
        ptr as *mut c_void,
        Some(on_tablist_removed),
      );
    }
  }
}

extern "C" fn on_tablist_added(obj: *mut c_void, id: c_int) {
  let module = obj as *mut TabListModule;
  let module = unsafe { &mut *module };
  let id = id as usize;

  // print(format!("add {}", id));

  module.entries.insert(id, TabListEntry::from_id(id));
}

extern "C" fn on_tablist_changed(obj: *mut c_void, id: c_int) {
  let module = obj as *mut TabListModule;
  let module = unsafe { &mut *module };
  let id = id as usize;

  // print(format!("changed {}", id));

  module.entries.insert(id, TabListEntry::from_id(id));
}

extern "C" fn on_tablist_removed(obj: *mut c_void, id: c_int) {
  let module = obj as *mut TabListModule;
  let module = unsafe { &mut *module };
  let id = id as usize;

  // print(format!("removed {}", id));

  module.entries.remove(&id);
}
