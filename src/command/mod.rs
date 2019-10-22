use crate::{
  chatsounds::{CHATSOUNDS, VOLUME_NORMAL},
  events::block_future,
  option,
  printer::Printer,
};
use classicube_sys::{Commands_Register, OwnedChatCommand};
use std::{cell::RefCell, convert::TryInto, os::raw::c_int};

pub const VOLUME_SETTING_NAME: &str = "chatsounds-volume";
const VOLUME_COMMAND_HELP: &str = "&a/client chatsounds volume [volume] &e(Default 1.0)";
const SH_COMMAND_HELP: &str = "&a/client chatsounds sh";

thread_local! {
  static COMMAND: RefCell<OwnedChatCommand> = RefCell::new(OwnedChatCommand::new(
    "Chatsounds",
    c_command_callback,
    false,
    vec![VOLUME_COMMAND_HELP,SH_COMMAND_HELP],
  ));
}

unsafe extern "C" fn c_command_callback(args: *const classicube_sys::String, args_count: c_int) {
  let args = std::slice::from_raw_parts(args, args_count.try_into().unwrap());
  let args: Vec<String> = args.iter().map(|cc_string| cc_string.to_string()).collect();

  block_future(async move {
    command_callback(args).await;
  });
}

async fn command_callback(args: Vec<String>) {
  let args: Vec<&str> = args.iter().map(|s| s.as_ref()).collect();

  match args.as_slice() {
    ["volume"] => {
      if let Some(chatsounds) = CHATSOUNDS.lock().await.as_mut() {
        let current_volume = chatsounds.volume() / VOLUME_NORMAL;
        Printer::chat_add(format!(
          "{} (Currently {})",
          VOLUME_COMMAND_HELP, current_volume
        ));
      }
    }

    ["volume", volume] => {
      let volume_maybe: Result<f32, _> = volume.parse();
      match volume_maybe {
        Ok(volume) => {
          Printer::chat_add(format!("&eSetting volume to {}", volume));

          if let Some(chatsounds) = CHATSOUNDS.lock().await.as_mut() {
            chatsounds.set_volume(VOLUME_NORMAL * volume);
          }
          option::set(VOLUME_SETTING_NAME, format!("{}", volume));
        }
        Err(e) => {
          Printer::chat_add(format!("&c{}", e));
        }
      }
    }

    ["sh"] => {
      if let Some(chatsounds) = CHATSOUNDS.lock().await.as_mut() {
        chatsounds.stop_all();
      }
    }

    _ => {
      if let Some(chatsounds) = CHATSOUNDS.lock().await.as_mut() {
        let current_volume = chatsounds.volume() / VOLUME_NORMAL;
        Printer::chat_add(format!(
          "{} (Currently {})",
          VOLUME_COMMAND_HELP, current_volume
        ));
        Printer::chat_add(SH_COMMAND_HELP);
      }
    }
  }
}

pub fn load() {
  COMMAND.with(|owned_command| unsafe {
    Commands_Register(&mut owned_command.borrow_mut().command);
  });
}

pub fn unload() {}
