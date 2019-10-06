use chatsounds::*;
use classicube::{
  ChatEvents, Chat_Add, Event_RegisterChat, IGameComponent, MsgType_MSG_TYPE_NORMAL,
};
use lazy_static::lazy_static;
use std::{
  os::raw::{c_int, c_void},
  ptr,
  sync::Mutex,
  thread,
};

lazy_static! {
  static ref CHATSOUNDS: Mutex<Option<Chatsounds>> = Mutex::new(None);
}

fn print<T: Into<String>>(s: T) {
  let s = s.into();
  unsafe {
    let cc_str = classicube::String::from_string(s);
    Chat_Add(&cc_str);
  };
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

  let mut msg = msg.to_string();

  if let Some(pos) = msg.rfind("&f") {
    let text = msg.split_off(pos + 2);

    // use winapi::um::winuser::{MessageBoxA, MB_ICONINFORMATION, MB_OK};

    // let lp_text = std::ffi::CString::new(format!("{}", msg)).unwrap();
    // let lp_caption = std::ffi::CString::new(format!("{}", text)).unwrap();

    // unsafe {
    //   MessageBoxA(
    //     std::ptr::null_mut(),
    //     lp_text.as_ptr(),
    //     lp_caption.as_ptr(),
    //     MB_OK | MB_ICONINFORMATION,
    //   );
    // }
    thread::spawn(move || {
      if let Some(chatsounds) = CHATSOUNDS.lock().unwrap().as_mut() {
        let text = text.trim();
        let mut found = chatsounds.find(text);
        if !found.is_empty() {
          let mut loaded = found[0].download();
          loaded.play(&chatsounds.sink);
        }
      }
    });
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

  let chatsounds = Chatsounds::new(vec![
    (
      "Metastruct/garrysmod-chatsounds".to_string(),
      "sound/chatsounds/autoadd".to_string(),
    ),
    (
      "PAC3-Server/chatsounds".to_string(),
      "sounds/chatsounds".to_string(),
    ),
  ]);

  chatsounds.sink.set_volume(0.1);

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
