// interact directly with the C functions, converting primitives to rust types

use super::{IncomingEvent, EVENT_HANDLER_MODULE};
use crate::modules::EventHandlerModule;
use classicube_sys::{Key_, MsgType, ScheduledTask};
use detour::static_detour;
use std::{
  cell::Cell,
  os::raw::{c_int, c_void},
};

thread_local! {
  pub static SIMULATING: Cell<bool> = Cell::new(false);
}

static_detour! {
  pub static TICK_DETOUR: unsafe extern "C" fn(*mut ScheduledTask);
}

pub fn tick_detour(task: *mut ScheduledTask) {
  unsafe {
    // call original Server.Tick
    TICK_DETOUR.call(task);
  }

  EVENT_HANDLER_MODULE.with(|maybe_ptr| {
    if let Some(ptr) = maybe_ptr.get() {
      let module = unsafe { &mut *ptr };

      module.handle_incoming_event(IncomingEvent::Tick);
      module.handle_outgoing_events();
    }
  });
}

pub extern "C" fn on_chat_received(
  obj: *mut c_void,
  full_msg: *const classicube_sys::String,
  msg_type: c_int,
) {
  let module = obj as *mut EventHandlerModule;
  let module = unsafe { &mut *module };

  if module.simulating {
    return;
  }

  let full_msg = if full_msg.is_null() {
    return;
  } else {
    unsafe { *full_msg }
  };

  let full_msg = full_msg.to_string();

  let msg_type: MsgType = msg_type as MsgType;

  module.handle_incoming_event(IncomingEvent::ChatReceived(full_msg, msg_type));
  module.handle_outgoing_events();
}

#[inline]
fn on_input_down(obj: *mut c_void, key: c_int, repeat: bool) {
  let module = obj as *mut EventHandlerModule;
  let module = unsafe { &mut *module };

  if module.simulating {
    return;
  }

  let key = key as Key_;

  module.handle_incoming_event(IncomingEvent::InputDown(key, repeat));
  module.handle_outgoing_events();
}

#[cfg(target_os = "macos")]
pub extern "C" fn c_on_input_down(obj: *mut c_void, key: c_int, repeat: bool) {
  on_input_down(obj, key, repeat)
}

#[cfg(not(target_os = "macos"))]
pub extern "C" fn c_on_input_down(obj: *mut c_void, key: c_int, repeat: u8) {
  on_input_down(obj, key, repeat != 0)
}

pub extern "C" fn on_input_up(obj: *mut c_void, key: c_int) {
  let module = obj as *mut EventHandlerModule;
  let module = unsafe { &mut *module };

  if module.simulating {
    return;
  }

  let key = key as Key_;

  module.handle_incoming_event(IncomingEvent::InputUp(key));
  module.handle_outgoing_events();
}

pub extern "C" fn on_input_press(obj: *mut c_void, key: c_int) {
  let module = obj as *mut EventHandlerModule;
  let module = unsafe { &mut *module };

  if module.simulating {
    return;
  }

  let key = key as u8 as char;

  module.handle_incoming_event(IncomingEvent::InputPress(key));
  module.handle_outgoing_events();
}
