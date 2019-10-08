use crate::{command, events, option, printer::PRINTER};
use classicube::{ScheduledTask, Server};
use detour::static_detour;

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
}

pub fn unload() {
  events::unload();

  option::unload();

  unsafe {
    TICK_DETOUR.disable().unwrap();
  }

  crate::chatsounds::unload();
}
