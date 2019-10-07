use chatsounds::*;
use classicube::{
  ChatEvents, Chat_AddOf, Event_RegisterChat, Event_RegisterInt, Event_UnregisterChat,
  Event_UnregisterInt, IGameComponent, InputEvents, MsgType_MSG_TYPE_BOTTOMRIGHT_1,
  MsgType_MSG_TYPE_BOTTOMRIGHT_2, MsgType_MSG_TYPE_BOTTOMRIGHT_3, MsgType_MSG_TYPE_NORMAL,
  ScheduledTask, Server,
};
use detour::static_detour;
use lazy_static::lazy_static;
use parking_lot::{Mutex, Once};
use rand::seq::SliceRandom;
use std::{
  os::raw::{c_int, c_void},
  ptr,
  sync::mpsc::{channel, Receiver, Sender},
  thread,
};

lazy_static! {
  static ref CHATSOUNDS: Mutex<Option<Chatsounds>> = Mutex::new(None);
}

static_detour! {
  static TICK_DETOUR: unsafe extern "C" fn(*mut ScheduledTask);
}

static mut PRINT_RECEIVER: Option<Receiver<String>> = None;
static mut PRINT_SENDER: Option<Sender<String>> = None;

fn print<T: Into<String>>(s: T) {
  unsafe {
    let sender = PRINT_SENDER.as_ref().unwrap();
    sender.send(s.into()).unwrap();
  }
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

static mut REGISTERED: bool = false;

extern "C" fn init() {
  unsafe {
    Event_RegisterChat(
      &mut ChatEvents.ChatReceived,
      ptr::null_mut(),
      Some(chat_on_received),
    );

    Event_RegisterInt(&mut InputEvents.Press, ptr::null_mut(), Some(on_key_press));

    REGISTERED = true;
  }
}

extern "C" fn free() {
  unsafe {
    if REGISTERED {
      Event_UnregisterChat(
        &mut ChatEvents.ChatReceived,
        ptr::null_mut(),
        Some(chat_on_received),
      );

      Event_UnregisterInt(&mut InputEvents.Press, ptr::null_mut(), Some(on_key_press));

      REGISTERED = false;
    }
  }

  *CHATSOUNDS.lock() = None;
}

fn tick_detour(task: *mut ScheduledTask) {
  unsafe {
    // call original Server.Tick
    TICK_DETOUR.call(task);

    for s in PRINT_RECEIVER.as_ref().unwrap().try_iter() {
      let length = s.len() as u16;
      let capacity = s.len() as u16;

      let c_str = std::ffi::CString::new(s).unwrap();

      let buffer = c_str.as_ptr() as *mut i8;

      let cc_str = classicube::String {
        buffer,
        length,
        capacity,
      };

      Chat_AddOf(&cc_str, MsgType_MSG_TYPE_BOTTOMRIGHT_1);
    }
  }
}

static START: Once = Once::new();

extern "C" fn on_map_loaded() {
  START.call_once(|| {
    unsafe {
      let (sender, receiver) = channel();
      PRINT_SENDER = Some(sender);
      PRINT_RECEIVER = Some(receiver);

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

#[no_mangle]
pub static Plugin_ApiVersion: c_int = 1;

#[no_mangle]
pub static mut Plugin_Component: IGameComponent = IGameComponent {
  Init: Some(init),
  Free: Some(free),
  Reset: Some(free),
  OnNewMap: None,
  OnNewMapLoaded: Some(on_map_loaded),
  next: ptr::null_mut(),
};
