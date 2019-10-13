use crate::{chat::CHAT, chatsounds::CHATSOUNDS, thread};
use classicube_sys::{
  ChatEvents, Event_RaiseInput, Event_RaiseInt, Event_RegisterChat, Event_RegisterInt,
  Event_UnregisterChat, Event_UnregisterInt, InputEvents, Key_, Key__KEY_BACKSPACE, MsgType,
  MsgType_MSG_TYPE_NORMAL,
};
use detour::static_detour;
use rand::seq::SliceRandom;
use std::{
  cell::{Cell, RefCell},
  os::raw::{c_int, c_void},
  ptr,
};

fn handle_chat_message<S: Into<String>>(full_msg: S) {
  let mut full_msg = full_msg.into();

  if let Some(pos) = full_msg.rfind("&f") {
    let msg = full_msg.split_off(pos + 2);

    thread::spawn("chatsounds handle message", move || {
      if let Some(chatsounds) = CHATSOUNDS.lock().as_mut() {
        let msg = msg.trim();

        if msg.to_lowercase() == "sh" {
          chatsounds.stop_all();
          return;
        }

        if let Some(sounds) = chatsounds.get(msg) {
          let mut rng = rand::thread_rng();

          if let Some(sound) = sounds.choose(&mut rng).cloned() {
            chatsounds.play(&sound);
          }
        }
      }
    });
  }
}

thread_local! {
  static CHAT_LAST: RefCell<Option<String>> = RefCell::new(None);
  pub static TYPE: RefCell<Option<String>> = RefCell::new(None);
}

extern "C" fn on_chat_received(
  _obj: *mut c_void,
  full_msg: *const classicube_sys::String,
  msg_type: c_int,
) {
  // TODO remove all &1 colors, then first find colon

  let msg_type: MsgType = msg_type as MsgType;

  if msg_type != MsgType_MSG_TYPE_NORMAL {
    return;
  }
  let full_msg = if full_msg.is_null() {
    return;
  } else {
    unsafe { *full_msg }
  };

  let mut full_msg = full_msg.to_string();

  CHAT_LAST.with(|chat_last| {
    let mut chat_last = chat_last.borrow_mut();

    if !full_msg.starts_with("> &f") {
      *chat_last = Some(full_msg.clone());
    } else if let Some(chat_last) = &*chat_last {
      // we're a continue message
      full_msg = full_msg.split_off(4); // skip "> &f"

      // most likely there's a space
      // the server trims the first line :(
      // TODO try both messages? with and without the space?
      full_msg = format!("{} {}", chat_last, full_msg);
    }
  });

  handle_chat_message(&full_msg);
}

static_detour! {
  static KEY_DOWN_DETOUR: unsafe extern "C" fn(*mut c_void, c_int, u8);
}

fn key_down_detour(obj: *mut c_void, key: c_int, repeat: u8) {
  let should_handle_input = {
    let key: Key_ = key as Key_;
    let repeat = repeat != 0;

    on_key_down(obj, key, repeat)
  };

  if should_handle_input {
    unsafe {
      // call original
      KEY_DOWN_DETOUR.call(obj, key, repeat);
    }
  }
}

thread_local! {
  static SIMULATING: Cell<bool> = Cell::new(false);
}

fn on_key_down(_obj: *mut c_void, key: Key_, repeat: bool) -> bool {
  if SIMULATING.with(|simulating| simulating.get()) {
    return true;
  }

  CHAT.with(|chat| {
    let mut chat = chat.borrow_mut();
    chat.handle_key_down(key, repeat)
  })
}

extern "C" fn on_key_press(_obj: *mut c_void, key: c_int) {
  if SIMULATING.with(|simulating| simulating.get()) {
    return;
  }

  CHAT.with(|chat| {
    let mut chat = chat.borrow_mut();
    chat.handle_key_press(key);
  });
}

pub fn simulate_char(chr: u8) {
  SIMULATING.with(|simulating| {
    simulating.set(true);
  });

  unsafe {
    Event_RaiseInt(&mut InputEvents.Press, c_int::from(chr));
  }

  SIMULATING.with(|simulating| {
    simulating.set(false);
  });
}

pub fn simulate_key(key: Key_) {
  SIMULATING.with(|simulating| {
    simulating.set(true);
  });

  unsafe {
    Event_RaiseInput(&mut InputEvents.Down, key as _, false);
    Event_RaiseInt(&mut InputEvents.Up, key as _);
  }

  SIMULATING.with(|simulating| {
    simulating.set(false);
  });
}

pub fn load() {
  unsafe {
    KEY_DOWN_DETOUR
      .initialize(InputEvents.Down.Handlers[0].unwrap(), key_down_detour)
      .unwrap();
    KEY_DOWN_DETOUR.enable().unwrap();

    Event_RegisterChat(
      &mut ChatEvents.ChatReceived,
      ptr::null_mut(),
      Some(on_chat_received),
    );

    // Event_RegisterInput(&mut InputEvents.Down, ptr::null_mut(), Some(on_key_down));
    Event_RegisterInt(&mut InputEvents.Press, ptr::null_mut(), Some(on_key_press));
  }
}

pub fn unload() {
  unsafe {
    let _ = KEY_DOWN_DETOUR.disable();

    Event_UnregisterChat(
      &mut ChatEvents.ChatReceived,
      ptr::null_mut(),
      Some(on_chat_received),
    );

    // Event_UnregisterInput(&mut InputEvents.Down, ptr::null_mut(), Some(on_key_down));
    Event_UnregisterInt(&mut InputEvents.Press, ptr::null_mut(), Some(on_key_press));
  }
}
