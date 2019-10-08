use crate::{command, events, option, printer::PRINTER};
use classicube::{ScheduledTask, Server};
use detour::static_detour;
use parking_lot::Once;

static LOAD_ONCE: Once = Once::new();

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
  LOAD_ONCE.call_once(|| {
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
  }); // Once
}

pub fn unload() {
  events::unload();

  unsafe {
    TICK_DETOUR.disable().unwrap();
  }

  crate::chatsounds::unload();
}
