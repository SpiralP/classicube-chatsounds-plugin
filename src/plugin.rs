use crate::{command, entities, events, option, printer::PRINTER, tablist};
use classicube_sys::{ScheduledTask, Server, String_AppendConst};
use detour::static_detour;
use std::{cell::Cell, ffi::CString};

thread_local! {
  static APP_NAME: Cell<Option<CString>> = Cell::new(None);
}

static_detour! {
  static TICK_DETOUR: unsafe extern "C" fn(*mut ScheduledTask);
}

fn tick_detour(task: *mut ScheduledTask) {
  unsafe {
    // call original Server.Tick
    TICK_DETOUR.call(task);
  }

  PRINTER.lock().flush();
  // TODO SPATIAL_SOUNDS.lock().update();
}

pub fn load() {
  events::load();
  command::load();
  option::load();
  entities::load();
  tablist::load();

  unsafe {
    if let Some(tick_original) = Server.Tick {
      TICK_DETOUR.initialize(tick_original, tick_detour).unwrap();
      TICK_DETOUR.enable().unwrap();
    }
  }

  crate::chatsounds::load();

  APP_NAME.with(|app_name| {
    let append_app_name = CString::new(format!(" +cs{}", env!("CARGO_PKG_VERSION"))).unwrap();

    let c_str = append_app_name.as_ptr();
    app_name.set(Some(append_app_name));

    unsafe {
      String_AppendConst(&mut Server.AppName, c_str);
    }
  });
}

pub fn unload() {
  events::unload();
  option::unload();
  entities::unload();
  tablist::unload();

  unsafe {
    let _ = TICK_DETOUR.disable();
  }

  crate::chatsounds::unload();

  APP_NAME.with(|app_name| {
    app_name.set(None);
  });
}

// pub fn reset() {
//   ENTITIES.with(|entities| {
//     let mut entities = entities.borrow_mut();
//     entities.clear();
//   });
// }

// pub fn on_new_map() {
//   ENTITIES.with(|entities| {
//     let mut entities = entities.borrow_mut();
//     entities.clear();
//   });
// }

// pub fn on_new_map_loaded() {
//   ENTITIES.with(|entities| {
//     let mut entities = entities.borrow_mut();
//     *entities = get_all_entities();
//   });
// }
