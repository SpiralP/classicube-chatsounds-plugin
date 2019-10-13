use crate::{chat::CHAT, chatsounds::CHATSOUNDS, thread};
use classicube_sys::{
  ChatEvents, Event_RaiseInput, Event_RaiseInt, Event_RegisterChat, Event_RegisterInput,
  Event_RegisterInt, Event_UnregisterChat, Event_UnregisterInput, Event_UnregisterInt, InputEvents,
  Key_, MsgType, MsgType_MSG_TYPE_NORMAL,
};
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

  CHAT_LAST.with(|maybe_chat_last| {
    let mut maybe_chat_last = maybe_chat_last.borrow_mut();

    // TODO remove all &1 colors, then first find colon

    if !full_msg.starts_with("> &f") {
      *maybe_chat_last = Some(full_msg.clone());
    } else if let Some(chat_last) = &*maybe_chat_last {
      // we're a continue message
      full_msg = full_msg.split_off(4); // skip "> &f"

      // most likely there's a space
      // the server trims the first line :(
      // TODO try both messages? with and without the space?
      full_msg = format!("{} {}", chat_last, full_msg);
      *maybe_chat_last = Some(full_msg.clone());
    }
  });

  handle_chat_message(&full_msg);
}

thread_local! {
  static SIMULATING: Cell<bool> = Cell::new(false);
}

extern "C" fn on_key_down(_obj: *mut c_void, key: c_int, repeat: u8) {
  if SIMULATING.with(|simulating| simulating.get()) {
    return;
  }

  CHAT.with(|chat| {
    let key: Key_ = key as Key_;
    let repeat = repeat != 0;

    let mut chat = chat.borrow_mut();
    chat.handle_key_down(key, repeat);
  });
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
    Event_RegisterChat(
      &mut ChatEvents.ChatReceived,
      ptr::null_mut(),
      Some(on_chat_received),
    );

    Event_RegisterInput(&mut InputEvents.Down, ptr::null_mut(), Some(on_key_down));
    Event_RegisterInt(&mut InputEvents.Press, ptr::null_mut(), Some(on_key_press));
  }
}

pub fn unload() {
  unsafe {
    Event_UnregisterChat(
      &mut ChatEvents.ChatReceived,
      ptr::null_mut(),
      Some(on_chat_received),
    );

    Event_UnregisterInput(&mut InputEvents.Down, ptr::null_mut(), Some(on_key_down));
    Event_UnregisterInt(&mut InputEvents.Press, ptr::null_mut(), Some(on_key_press));
  }
}
