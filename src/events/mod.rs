//! interact directly with the C functions, converting primitives to rust types

mod handlers;

pub use self::handlers::*;
use classicube_sys::{
  ChatEvents, Event_RegisterChat, Event_RegisterInput, Event_RegisterInt, Event_UnregisterChat,
  Event_UnregisterInput, Event_UnregisterInt, InputEvents, Key_, MsgType, ScheduledTask, Server,
};
use detour::static_detour;
use std::{
  cell::Cell,
  os::raw::{c_int, c_void},
  ptr,
};

thread_local! {
  pub static SIMULATING: Cell<bool> = Cell::new(false);
}

static_detour! {
  static TICK_DETOUR: unsafe extern "C" fn(*mut ScheduledTask);
}

fn tick_detour(task: *mut ScheduledTask) {
  unsafe {
    // call original Server.Tick
    TICK_DETOUR.call(task);
  }

  handle_incoming_event(IncomingEvent::Tick);
  handle_outgoing_events();
}

extern "C" fn on_chat_received(
  _obj: *mut c_void,
  full_msg: *const classicube_sys::String,
  msg_type: c_int,
) {
  if SIMULATING.with(|simulating| simulating.get()) {
    return;
  }

  let full_msg = if full_msg.is_null() {
    return;
  } else {
    unsafe { *full_msg }
  };

  let full_msg = full_msg.to_string();

  let msg_type: MsgType = msg_type as MsgType;

  handle_incoming_event(IncomingEvent::ChatReceived(full_msg, msg_type));
  handle_outgoing_events();
}

#[inline]
fn on_input_down(key: Key_, repeat: bool) {
  if SIMULATING.with(|simulating| simulating.get()) {
    return;
  }

  let key = key as Key_;

  handle_incoming_event(IncomingEvent::InputDown(key, repeat));
  handle_outgoing_events();
}

#[cfg(target_os = "macos")]
extern "C" fn c_on_input_down(_obj: *mut c_void, key: c_int, repeat: bool) {
  on_input_down(key, repeat)
}

#[cfg(not(target_os = "macos"))]
extern "C" fn c_on_input_down(_obj: *mut c_void, key: c_int, repeat: u8) {
  on_input_down(key, repeat != 0)
}

extern "C" fn on_input_up(_obj: *mut c_void, key: c_int) {
  if SIMULATING.with(|simulating| simulating.get()) {
    return;
  }

  let key = key as Key_;

  handle_incoming_event(IncomingEvent::InputUp(key));
  handle_outgoing_events();
}

extern "C" fn on_input_press(_obj: *mut c_void, key: c_int) {
  if SIMULATING.with(|simulating| simulating.get()) {
    return;
  }

  let key = key as u8 as char;

  handle_incoming_event(IncomingEvent::InputPress(key));
  handle_outgoing_events();
}

pub fn load() {
  self::handlers::load();

  unsafe {
    Event_RegisterChat(
      &mut ChatEvents.ChatReceived,
      ptr::null_mut(),
      Some(on_chat_received),
    );

    Event_RegisterInput(
      &mut InputEvents.Down,
      ptr::null_mut(),
      Some(c_on_input_down),
    );
    Event_RegisterInt(&mut InputEvents.Up, ptr::null_mut(), Some(on_input_up));
    Event_RegisterInt(
      &mut InputEvents.Press,
      ptr::null_mut(),
      Some(on_input_press),
    );
  }

  unsafe {
    if let Some(tick_original) = Server.Tick {
      TICK_DETOUR.initialize(tick_original, tick_detour).unwrap();
      TICK_DETOUR.enable().unwrap();
    }
  }
}

pub fn unload() {
  unsafe {
    let _ = TICK_DETOUR.disable();
  }

  unsafe {
    Event_UnregisterChat(
      &mut ChatEvents.ChatReceived,
      ptr::null_mut(),
      Some(on_chat_received),
    );

    Event_UnregisterInput(
      &mut InputEvents.Down,
      ptr::null_mut(),
      Some(c_on_input_down),
    );
    Event_UnregisterInt(&mut InputEvents.Up, ptr::null_mut(), Some(on_input_up));
    Event_UnregisterInt(
      &mut InputEvents.Press,
      ptr::null_mut(),
      Some(on_input_press),
    );
  }

  self::handlers::unload();
}
