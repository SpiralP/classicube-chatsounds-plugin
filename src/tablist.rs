use classicube_sys::{
  Event_RegisterInt, Event_UnregisterInt, StringsBuffer_UNSAFE_Get, TabList, TabListEvents,
};
use std::{
  cell::RefCell,
  collections::HashMap,
  os::raw::{c_int, c_void},
  ptr,
};

thread_local! {
  pub static TABLIST: RefCell<HashMap<usize, TabListEntry>> = RefCell::new(HashMap::new());
}

#[derive(Debug, Clone)]
pub struct TabListEntry {
  pub id: usize,
  pub real_name: String,
  pub nick_name: String,
  pub group: String,
}

fn tablist_get_real_name(id: usize) -> String {
  // or "Player"
  unsafe {
    StringsBuffer_UNSAFE_Get(
      &mut TabList._buffer,
      c_int::from(TabList.NameOffsets[id] - 3),
    )
  }
  .to_string()
}

fn tablist_get_nick_name(id: usize) -> String {
  // or "Text"
  unsafe {
    StringsBuffer_UNSAFE_Get(
      &mut TabList._buffer,
      c_int::from(TabList.NameOffsets[id] - 2),
    )
  }
  .to_string()
}

fn tablist_get_group(id: usize) -> String {
  unsafe {
    StringsBuffer_UNSAFE_Get(
      &mut TabList._buffer,
      c_int::from(TabList.NameOffsets[id] - 1),
    )
  }
  .to_string()
}

fn get_tablist_entry(id: usize) -> TabListEntry {
  TabListEntry {
    id,
    real_name: tablist_get_real_name(id),
    nick_name: tablist_get_nick_name(id),
    group: tablist_get_group(id),
  }
}

extern "C" fn on_tablist_added(_obj: *mut c_void, id: c_int) {
  let id = id as usize;

  // print(format!("add {}", id));

  TABLIST.with(|tablist| {
    let mut tablist = tablist.borrow_mut();

    tablist.insert(id, get_tablist_entry(id));
  });
}

extern "C" fn on_tablist_changed(_obj: *mut c_void, id: c_int) {
  let id = id as usize;

  // print(format!("changed {}", id));

  TABLIST.with(|tablist| {
    let mut tablist = tablist.borrow_mut();

    tablist.insert(id, get_tablist_entry(id));
  });
}

extern "C" fn on_tablist_removed(_obj: *mut c_void, id: c_int) {
  let id = id as usize;

  // print(format!("removed {}", id));

  TABLIST.with(|tablist| {
    let mut tablist = tablist.borrow_mut();

    tablist.remove(&id);
  });
}

pub fn load() {
  unsafe {
    Event_RegisterInt(
      &mut TabListEvents.Added,
      ptr::null_mut(),
      Some(on_tablist_added),
    );
    Event_RegisterInt(
      &mut TabListEvents.Changed,
      ptr::null_mut(),
      Some(on_tablist_changed),
    );
    Event_RegisterInt(
      &mut TabListEvents.Removed,
      ptr::null_mut(),
      Some(on_tablist_removed),
    );
  }
}

pub fn unload() {
  unsafe {
    Event_UnregisterInt(
      &mut TabListEvents.Added,
      ptr::null_mut(),
      Some(on_tablist_added),
    );
    Event_UnregisterInt(
      &mut TabListEvents.Changed,
      ptr::null_mut(),
      Some(on_tablist_changed),
    );
    Event_UnregisterInt(
      &mut TabListEvents.Removed,
      ptr::null_mut(),
      Some(on_tablist_removed),
    );
  }
}
