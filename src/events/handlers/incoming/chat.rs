use crate::{chat::Chat, chatsounds};
use classicube_sys::{MsgType, MsgType_MSG_TYPE_NORMAL};
use futures::lock::Mutex;
use lazy_static::lazy_static;

lazy_static! {
  static ref CHAT_LAST: Mutex<Option<String>> = Mutex::new(None);
  pub static ref CHAT: Mutex<Chat> = Mutex::new(Chat::new());
}

pub async fn handle_chat_received(mut full_msg: String, msg_type: MsgType) {
  if msg_type != MsgType_MSG_TYPE_NORMAL {
    return;
  }

  let mut maybe_chat_last = CHAT_LAST.lock().await;

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

  chatsounds::handle_chat_message(&full_msg).await;
}
