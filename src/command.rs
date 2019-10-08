use crate::{chatsounds::CHATSOUNDS, printer::Printer};
use arrayvec::ArrayVec;
use classicube::{ChatCommand, Commands_Register};
use std::{convert::TryInto, ffi::CString, os::raw::c_int, ptr};

pub static mut COMMAND: Option<OwnedChatCommand> = None;
const VOLUME_COMMAND_HELP: &str = "&a/client chatsounds volume [volume] &e(Default 1.0)";
const VOLUME_NORMAL: f32 = 0.1;

pub struct OwnedChatCommand {
  pub name: CString,
  pub help: Vec<CString>,
  pub command: ChatCommand,
}

impl OwnedChatCommand {
  pub fn new(
    name: &'static str,
    execute: unsafe extern "C" fn(args: *const classicube::String, argsCount: c_int),
    singleplayer_only: bool,
    mut help: Vec<&'static str>,
  ) -> Self {
    let name = CString::new(name).unwrap();

    let help: Vec<CString> = help.drain(..).map(|s| CString::new(s).unwrap()).collect();

    let command = ChatCommand {
      Name: name.as_ptr(),
      Execute: Some(execute),
      SingleplayerOnly: if singleplayer_only { 1 } else { 0 },
      Help: {
        let mut array: ArrayVec<[*const ::std::os::raw::c_char; 5usize]> =
          help.iter().map(|cstr| cstr.as_ptr()).collect();

        while !array.is_full() {
          array.push(ptr::null());
        }

        array.into_inner().unwrap()
      },
      next: ptr::null_mut(),
    };

    Self {
      name,
      help,
      command,
    }
  }
}

unsafe extern "C" fn command_callback(args: *const classicube::String, args_count: c_int) {
  let args = std::slice::from_raw_parts(args, args_count.try_into().unwrap());
  let args: Vec<String> = args.iter().map(|cc_string| cc_string.to_string()).collect();
  let args: Vec<&str> = args.iter().map(|s| s.as_ref()).collect();

  match args.as_slice() {
    ["volume"] => {
      Printer::chat_add(VOLUME_COMMAND_HELP);
    }

    ["volume", volume] => {
      let volume_maybe: Result<f32, _> = volume.parse();
      match volume_maybe {
        Ok(volume) => {
          // TODO store in Options_xxx
          Printer::chat_add(format!("&eSetting volume to {}", volume));
          if let Some(chatsounds) = CHATSOUNDS.lock().as_mut() {
            chatsounds.set_volume(VOLUME_NORMAL * volume);
          }
        }
        Err(e) => {
          Printer::chat_add(format!("&c{}", e));
        }
      }
    }

    _ => {
      Printer::chat_add(VOLUME_COMMAND_HELP);
      // ...rest
    }
  }
}

pub fn load() {
  unsafe {
    COMMAND = Some(OwnedChatCommand::new(
      "Chatsounds",
      command_callback,
      false,
      vec![VOLUME_COMMAND_HELP],
    ));

    Commands_Register(&mut COMMAND.as_mut().unwrap().command);
  }
}
