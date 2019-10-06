use chatsounds::*;
use classicube::{
  ChatEvents, Chat_Add, Event_RegisterChat, IGameComponent, MsgType_MSG_TYPE_NORMAL,
};
use lazy_static::lazy_static;
use std::{
  ffi::CString,
  os::raw::{c_int, c_void},
  ptr,
  sync::Mutex,
};
use winapi::{
  shared::ntdef::NULL,
  um::winuser::{MessageBoxA, MB_ICONINFORMATION, MB_OK},
};

lazy_static! {
  static ref CHATSOUNDS: Mutex<Option<Chatsounds>> = Mutex::new(None);
}

extern "C" fn chat_on_received(_obj: *mut c_void, msg: *const classicube::String, msg_type: c_int) {
  if msg_type != MsgType_MSG_TYPE_NORMAL {
    return;
  }
  let msg = if msg.is_null() {
    return;
  } else {
    unsafe { *msg }
  };

  let msg = msg.to_string();

  if msg.contains("bap") {
    let s = String::from("ag bo");
    unsafe {
      let cc_str = classicube::String::from_string(s);
      Chat_Add(&cc_str);
    };

    if let Some(chatsounds) = CHATSOUNDS.lock().unwrap().as_mut() {
      let mut found = chatsounds.find("bap");
      let mut loaded = found[0].download();
      loaded.play(&chatsounds.device);
    }
  }
}

extern "C" fn init() {
  unsafe {
    Event_RegisterChat(
      &mut ChatEvents.ChatReceived,
      ptr::null_mut(),
      Some(chat_on_received),
    );
  }

  let chatsounds = Chatsounds::new(
    "Metastruct/garrysmod-chatsounds",
    "sound/chatsounds/autoadd",
  );

  *CHATSOUNDS.lock().unwrap() = Some(chatsounds);
}

#[test]
fn asdf() {
  init();
}

#[no_mangle]
pub static Plugin_ApiVersion: c_int = 1;

#[no_mangle]
pub static mut Plugin_Component: IGameComponent = IGameComponent {
  Init: Some(init),
  Free: None,
  Reset: None,
  OnNewMap: None,
  OnNewMapLoaded: None,
  next: ptr::null_mut(),
};
