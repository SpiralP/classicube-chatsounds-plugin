use crate::{command, events, events::TYPE, option, printer::PRINTER};
use classicube_sys::{
  Event_RaiseInput, Event_RaiseInt, InputEvents, Key__KEY_BACKSPACE, ScheduledTask, Server,
  String_AppendConst,
};
use detour::static_detour;
use std::{cell::Cell, convert::TryInto, ffi::CString, os::raw::c_int};

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

thread_local! {
  static APP_NAME: Cell<Option<CString>> = Cell::new(None);
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

  APP_NAME.with(|app_name| {
    let append_app_name = CString::new(format!(" + Test v{}", env!("CARGO_PKG_VERSION"))).unwrap();

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

  unsafe {
    let _ = TICK_DETOUR.disable();
  }

  crate::chatsounds::unload();

  APP_NAME.with(|app_name| {
    app_name.set(None);
  });
}
