use crate::{command, events, option, printer::PRINTER};
use classicube::{ScheduledTask, Server};
use detour::static_detour;
use std::cell::Cell;

thread_local! {
  static LOADED: Cell<bool> = Cell::new(false);
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
}

pub fn load() {
  let loaded = LOADED.with(|a| a.get());

  if !loaded {
    events::load();
    command::load();

    option::load();

    unsafe {
      if let Some(tick_original) = Server.Tick {
        TICK_DETOUR.initialize(tick_original, tick_detour).unwrap();
        TICK_DETOUR.enable().unwrap();
      }
    }

    crate::chatsounds::load();

    LOADED.with(|a| a.set(true));
  }
}

pub fn unload() {
  let loaded = LOADED.with(|a| a.get());

  if loaded {
    events::unload();

    unsafe {
      TICK_DETOUR.disable().unwrap();
    }

    crate::chatsounds::unload();
  }

  LOADED.with(|a| a.set(false));
}
