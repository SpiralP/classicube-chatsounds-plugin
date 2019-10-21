use crate::{chat::Chat, chatsounds};
use classicube_sys::{MsgType, MsgType_MSG_TYPE_NORMAL};
use std::cell::RefCell;

thread_local! {
  pub static CHAT: RefCell<Chat> = RefCell::new(Chat::new());
}

thread_local! {
  static CHAT_LAST: RefCell<Option<String>> = RefCell::new(None);
}

pub fn handle_chat_received(mut full_msg: String, msg_type: MsgType) {
  if msg_type != MsgType_MSG_TYPE_NORMAL {
    return;
  }

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

  chatsounds::handle_chat_message(&full_msg);
}
