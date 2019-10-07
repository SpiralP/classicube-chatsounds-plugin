use crate::{option, option::get_key_from_input_name, printer::Printer};
use chatsounds::Chatsounds;
use classicube::{
  ChatEvents, Event_RegisterChat, Event_RegisterInput, Event_UnregisterChat, Event_UnregisterInput,
  InputEvents, Key, Key__KEY_0, Key__KEY_9, Key__KEY_A, Key__KEY_BACKSPACE, Key__KEY_ESCAPE,
  Key__KEY_KP_ENTER, Key__KEY_SLASH, Key__KEY_SPACE, Key__KEY_TAB, Key__KEY_Z,
  MsgType_MSG_TYPE_NORMAL, ScheduledTask, Server,
};
use detour::static_detour;
use lazy_static::lazy_static;
use parking_lot::{Mutex, Once};
use rand::seq::SliceRandom;
use std::{
  fs,
  os::raw::{c_int, c_void},
  path::Path,
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
  static ref CHAT: Mutex<Chat> = Mutex::new(Chat::new());
}

pub struct Chat {
  open: bool,
  text: Vec<u8>,
}
impl Chat {
  pub fn new() -> Self {
    Self {
      text: Vec::new(),
      open: false,
    }
  }

  pub fn get_text(&self) -> String {
    String::from_utf8_lossy(&self.text)
      .to_string()
      .to_lowercase()
  }

  pub fn handle_key(&mut self, key: Key, repeat: bool) {
    if !repeat {
      if !self.open && (key == CHAT_KEY.unwrap_or(0) || key == Key__KEY_SLASH) {
        print("OPEN");

        self.open = true;
        self.text.clear();
        return;
      }

      if key == SEND_CHAT_KEY.unwrap_or(0) || key == Key__KEY_KP_ENTER || key == Key__KEY_ESCAPE {
        print("CLOSE");

        self.open = false;
        self.text.clear();
        return;
      }
    }

    if self.open {
      // TODO ' and other symbols!
      // TODO shift + 2 should be @?

      if (key >= Key__KEY_A && key <= Key__KEY_Z) || (key >= Key__KEY_0 && key <= Key__KEY_9) {
        let chr = key as u8;
        self.text.push(chr);
      } else if key == Key__KEY_BACKSPACE {
        self.text.pop();
      } else if key == Key__KEY_SPACE {
        self.text.push(b' ');
      }

      print(self.get_text());
    }
  }
}

fn print<T: Into<String>>(s: T) {
  PRINTER.lock().print(s)
}

extern "C" fn on_chat_received(
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

// TODO init these in load()
lazy_static! {
  static ref CHAT_KEY: Option<Key> =
    { option::get("key-Chat").and_then(|s| get_key_from_input_name(&s)) };
  static ref SEND_CHAT_KEY: Option<Key> =
    { option::get("key-SendChat").and_then(|s| get_key_from_input_name(&s)) };
}

extern "C" fn on_key_down(_obj: *mut c_void, key: Key, repeat: u8) {
  let mut chat = CHAT.lock();
  chat.handle_key(key, repeat != 0);

  if key == Key__KEY_TAB {
    print("autocomplete me baby");
  }
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
        Some(on_chat_received),
      );

      Event_RegisterInput(&mut InputEvents.Down, ptr::null_mut(), Some(on_key_down));
    }

    *events_registered = true;

    unsafe {
      if let Some(tick_original) = Server.Tick {
        TICK_DETOUR.initialize(tick_original, tick_detour).unwrap();
        TICK_DETOUR.enable().unwrap();
      }
    }

    print("Loading chatsounds...");

    if fs::metadata("plugins")
      .map(|meta| meta.is_dir())
      .unwrap_or(false)
    {
      let path = Path::new("plugins/chatsounds");
      fs::create_dir_all(path).unwrap();

      let chatsounds = Chatsounds::new(path);
      *CHATSOUNDS.lock() = Some(chatsounds);
    } else {
      panic!("UH OH");
    }

    // thread::spawn(move || {
    //   if let Some(chatsounds) = CHATSOUNDS.lock().as_mut() {
    //     print("Metastruct/garrysmod-chatsounds");
    //     chatsounds.load_github_api(
    //       "Metastruct/garrysmod-chatsounds".to_string(),
    //       "sound/chatsounds/autoadd".to_string(),
    //     );
    //   }

    //   if let Some(chatsounds) = CHATSOUNDS.lock().as_mut() {
    //     print("PAC3-Server/chatsounds");
    //     chatsounds.load_github_api(
    //       "PAC3-Server/chatsounds".to_string(),
    //       "sounds/chatsounds".to_string(),
    //     );
    //   }

    //   for folder in &[
    //     "csgo", "css", "ep1", "ep2", "hl2", "l4d", "l4d2", "portal", "tf2",
    //   ] {
    //     if let Some(chatsounds) = CHATSOUNDS.lock().as_mut() {
    //       print(format!("PAC3-Server/chatsounds-valve-games {}", folder));
    //       chatsounds.load_github_msgpack(
    //         "PAC3-Server/chatsounds-valve-games".to_string(),
    //         folder.to_string(),
    //       );
    //     }
    //   }

    //   print("done fetching sources");
    // });
  }); // Once
}

pub fn unload() {
  let mut events_registered = EVENTS_REGISTERED.lock();

  if *events_registered {
    unsafe {
      Event_UnregisterChat(
        &mut ChatEvents.ChatReceived,
        ptr::null_mut(),
        Some(on_chat_received),
      );

      Event_UnregisterInput(&mut InputEvents.Down, ptr::null_mut(), Some(on_key_down));
    }
  }

  *events_registered = false;

  *CHATSOUNDS.lock() = None;
}
