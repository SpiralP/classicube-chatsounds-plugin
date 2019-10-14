use crate::{
  chat::CHAT, chatsounds::CHATSOUNDS, entities::ENTITIES, helpers::remove_color, printer::print,
  tablist::TABLIST, thread,
};
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

thread_local! {
  pub static SIMULATING: Cell<bool> = Cell::new(false);
}

fn play_chatsound(entity_id: usize, sentence: String) {
  // TODO need the main thread tick to update positions

  ENTITIES.with(|entities| {
    let entities = entities.borrow();

    if let Some(entity) = entities.get(&entity_id) {
      //
    }
  });

  // TODO use 1 thread and a channel
  thread::spawn("chatsounds handle message", move || {
    if let Some(chatsounds) = CHATSOUNDS.lock().as_mut() {
      if sentence.to_lowercase() == "sh" {
        chatsounds.stop_all();
        return;
      }

      if let Some(sounds) = chatsounds.get(sentence) {
        let mut rng = rand::thread_rng();

        if let Some(sound) = sounds.choose(&mut rng).cloned() {
          chatsounds.play(&sound);
        }
      }
    }
  });
}

fn handle_chat_message<S: Into<String>>(full_msg: S) {
  // &]SpiralP: &faaa
  let full_msg = full_msg.into();

  // nickname_resolver_handle_message(full_msg.to_string());

  // find colon from the left
  if let Some(pos) = full_msg.find(": ") {
    // &]SpiralP
    let left = &full_msg[..pos]; // left without colon
                                 // &faaa
    let right = &full_msg[(pos + 2)..]; // right without colon

    if right.find(':').is_some() {
      // no colons in any chatsound, and we could have parsed nick wrong
      return;
    }

    // TODO title is [ ] before nick, team is < > before nick, also there are rank symbols?
    // &f┬ &f♂&6 Goodly: &fhi

    let full_nick = left.to_string();
    let colorless_text: String = remove_color(right.to_string()).trim().to_string();

    // lookup entity id from nick_name by using TabList
    let found_entity_id = TABLIST.with(|tablist| {
      let tablist = tablist.borrow();

      // try exact match
      tablist
        .iter()
        .find_map(|(id, entry)| {
          if entry.nick_name == full_nick {
            Some(*id)
          } else {
            None
          }
        })
        .or_else(|| {
          // match from the right, choose the one with most chars matched
          let mut id_positions: Vec<(usize, usize)> = tablist
            .iter()
            .filter_map(|(id, entry)| {
              // full_nick &g[&x&7___&g] &m___&0 Cjnator38
              // real_nick &g&m___&0 Cjnator38

              // remove color at beginning
              let (full_nick_color, full_nick) =
                if full_nick.len() >= 2 && full_nick.starts_with('&') {
                  full_nick.split_at(2)
                } else {
                  ("", full_nick.as_str())
                };
              let (real_nick_color, real_nick) =
                if entry.nick_name.len() >= 2 && entry.nick_name.starts_with('&') {
                  entry.nick_name.split_at(2)
                } else {
                  ("", entry.nick_name.as_str())
                };

              full_nick
                .rfind(&real_nick)
                .filter(|_| full_nick_color == real_nick_color)
                .map(|pos| (*id, pos))
            })
            .collect();

          id_positions.sort_unstable_by(|(id1, pos1), (id2, pos2)| {
            pos1
              .partial_cmp(pos2)
              .unwrap()
              .then_with(|| id1.partial_cmp(&id2).unwrap())
          });

          id_positions.first().map(|(id, _pos)| *id)
        })
    });

    if let Some(entity_id) = found_entity_id {
      print(format!("FOUND {} {}", entity_id, full_nick));

      play_chatsound(entity_id, colorless_text);
    } else {
      print(format!("not found {}", full_nick));
    }
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
  if SIMULATING.with(|simulating| simulating.get()) {
    return;
  }

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
extern "C" fn on_key_up(_obj: *mut c_void, key: c_int) {
  if SIMULATING.with(|simulating| simulating.get()) {
    return;
  }

  CHAT.with(|chat| {
    let key: Key_ = key as Key_;

    let mut chat = chat.borrow_mut();
    chat.handle_key_up(key);
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
    Event_RegisterInt(&mut InputEvents.Up, ptr::null_mut(), Some(on_key_up));
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
    Event_UnregisterInt(&mut InputEvents.Up, ptr::null_mut(), Some(on_key_up));
    Event_UnregisterInt(&mut InputEvents.Press, ptr::null_mut(), Some(on_key_press));
  }
}
