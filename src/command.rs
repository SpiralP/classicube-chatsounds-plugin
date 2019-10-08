use crate::{
  chatsounds::{CHATSOUNDS, VOLUME_NORMAL},
  option,
  printer::Printer,
};
use arrayvec::ArrayVec;
use classicube::{ChatCommand, Commands_Register};
use std::{cell::RefCell, convert::TryInto, ffi::CString, os::raw::c_int, ptr};

pub const VOLUME_SETTING_NAME: &str = "chatsounds-volume";
const VOLUME_COMMAND_HELP: &str = "&a/client chatsounds volume [volume] &e(Default 1.0)";

thread_local! {
  pub static COMMAND: RefCell<OwnedChatCommand> = RefCell::new(OwnedChatCommand::new(
    "Chatsounds",
    c_command_callback,
    false,
    vec![VOLUME_COMMAND_HELP],
  ));
}

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

unsafe extern "C" fn c_command_callback(args: *const classicube::String, args_count: c_int) {
  let args = std::slice::from_raw_parts(args, args_count.try_into().unwrap());
  let args: Vec<String> = args.iter().map(|cc_string| cc_string.to_string()).collect();

  command_callback(args);
}

fn command_callback(args: Vec<String>) {
  let args: Vec<&str> = args.iter().map(|s| s.as_ref()).collect();

  if let Some(chatsounds) = CHATSOUNDS.lock().as_mut() {
    let current_volume = chatsounds.volume() / VOLUME_NORMAL;

    match args.as_slice() {
      ["volume"] => {
        Printer::chat_add(format!(
          "{} (Currently {})",
          VOLUME_COMMAND_HELP, current_volume
        ));
      }

      ["volume", volume] => {
        let volume_maybe: Result<f32, _> = volume.parse();
        match volume_maybe {
          Ok(volume) => {
            Printer::chat_add(format!("&eSetting volume to {}", volume));

            chatsounds.set_volume(VOLUME_NORMAL * volume);
            option::set(VOLUME_SETTING_NAME, format!("{}", volume));
          }
          Err(e) => {
            Printer::chat_add(format!("&c{}", e));
          }
        }
      }

      _ => {
        Printer::chat_add(format!(
          "{} (Currently {})",
          VOLUME_COMMAND_HELP, current_volume
        ));
        // ...rest
      }
    }
  }
}

pub fn load() {
  COMMAND.with(|owned_command| unsafe {
    Commands_Register(&mut owned_command.borrow_mut().command);
  });
}
