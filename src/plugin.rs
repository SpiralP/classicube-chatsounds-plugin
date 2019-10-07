use crate::printer::Printer;
use chatsounds::Chatsounds;
use classicube::{
  ChatEvents, Event_RegisterChat, Event_RegisterInt, Event_UnregisterChat, Event_UnregisterInt,
  InputEvents, MsgType_MSG_TYPE_NORMAL, ScheduledTask, Server,
};
use detour::static_detour;
use lazy_static::lazy_static;
use parking_lot::{Mutex, Once};
use rand::seq::SliceRandom;
use std::{
  os::raw::{c_int, c_void},
  ptr, thread,
};

static LOAD_ONCE: Once = Once::new();

static_detour! {
  static TICK_DETOUR: unsafe extern "C" fn(*mut ScheduledTask);
}

lazy_static! {
  static ref CHATSOUNDS: Mutex<Option<Chatsounds>> = Mutex::new(None);
  static ref PRINTER: Mutex<Printer> = Mutex::new(Printer::new());
  static ref EVENTS_REGISTERED: Mutex<bool> = Mutex::new(false);
}

fn print<T: Into<String>>(s: T) {
  PRINTER.lock().print(s)
}

extern "C" fn chat_on_received(
  _obj: *mut c_void,
  full_msg: *const classicube::String,
  msg_type: c_int,
) {
  if msg_type != MsgType_MSG_TYPE_NORMAL {
    return;
  }
  let full_msg = if full_msg.is_null() {
    return;
  } else {
    unsafe { *full_msg }
  };

  let mut full_msg = full_msg.to_string();

  if let Some(pos) = full_msg.rfind("&f") {
    let msg = full_msg.split_off(pos + 2);

    thread::spawn(move || {
      if let Some(chatsounds) = CHATSOUNDS.lock().as_mut() {
        let msg = msg.trim();
        if msg.to_lowercase() == "sh" {
          chatsounds.stop_all();
          return;
        }

        let mut sounds = chatsounds.find(msg);
        let mut rng = rand::thread_rng();

        if let Some(sound) = sounds.choose_mut(&mut rng) {
          sound.play(&chatsounds.device, &mut chatsounds.sinks);
        }
      }
    });
  }
}

extern "C" fn on_key_press(_obj: *mut c_void, chr: c_int) {
  // print(format!("{:?}", chr));
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
    let mut events_registered = EVENTS_REGISTERED.lock();

    unsafe {
      Event_RegisterChat(
        &mut ChatEvents.ChatReceived,
        ptr::null_mut(),
        Some(chat_on_received),
      );

      Event_RegisterInt(&mut InputEvents.Press, ptr::null_mut(), Some(on_key_press));
    }

    *events_registered = true;

    unsafe {
      if let Some(tick_original) = Server.Tick {
        TICK_DETOUR.initialize(tick_original, tick_detour).unwrap();
        TICK_DETOUR.enable().unwrap();
      }
    }

    print("Loading chatsounds...");

    {
      let chatsounds = Chatsounds::new();
      *CHATSOUNDS.lock() = Some(chatsounds);
    }

    thread::spawn(move || {
      if let Some(chatsounds) = CHATSOUNDS.lock().as_mut() {
        print("Metastruct/garrysmod-chatsounds");
        chatsounds.load_github_api(
          "Metastruct/garrysmod-chatsounds".to_string(),
          "sound/chatsounds/autoadd".to_string(),
        );

        print("PAC3-Server/chatsounds");
        chatsounds.load_github_api(
          "PAC3-Server/chatsounds".to_string(),
          "sounds/chatsounds".to_string(),
        );

        for folder in &[
          "csgo", "css", "ep1", "ep2", "hl2", "l4d", "l4d2", "portal", "tf2",
        ] {
          print(format!("PAC3-Server/chatsounds-valve-games {}", folder));
          chatsounds.load_github_api(
            "PAC3-Server/chatsounds-valve-games".to_string(),
            folder.to_string(),
          );
        }
      }
    });
  }); // Once
}

pub fn unload() {
  let mut events_registered = EVENTS_REGISTERED.lock();

  if *events_registered {
    unsafe {
      Event_UnregisterChat(
        &mut ChatEvents.ChatReceived,
        ptr::null_mut(),
        Some(chat_on_received),
      );

      Event_UnregisterInt(&mut InputEvents.Press, ptr::null_mut(), Some(on_key_press));
    }
  }

  *events_registered = false;

  *CHATSOUNDS.lock() = None;
}
