use crate::{command, events, events::TYPE, option, printer::PRINTER};
use classicube_sys::{
  Event_RaiseInput, Event_RaiseInt, InputEvents, Key__KEY_BACKSPACE, ScheduledTask, Server,
};
use detour::static_detour;
use std::{convert::TryInto, os::raw::c_int};

static_detour! {
  static TICK_DETOUR: unsafe extern "C" fn(*mut ScheduledTask);
}

fn tick_detour(task: *mut ScheduledTask) {
  unsafe {
    // call original Server.Tick
    TICK_DETOUR.call(task);
  }

  PRINTER.lock().flush();

  TYPE.with(|text| {
    if let Some(text) = text.borrow_mut().take() {
      for _ in 0..256 {
        unsafe {
          Event_RaiseInput(
            &mut InputEvents.Down,
            Key__KEY_BACKSPACE.try_into().unwrap(),
            false,
          );
          Event_RaiseInt(&mut InputEvents.Up, Key__KEY_BACKSPACE.try_into().unwrap());
        }
      }

      for c in text.chars() {
        unsafe {
          Event_RaiseInt(&mut InputEvents.Press, c as c_int);
        }
      }
    }
  });
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
    let _ = TICK_DETOUR.disable();
  }

  crate::chatsounds::unload();
}
