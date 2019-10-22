mod chat;
mod tick;

use self::{
  chat::{handle_chat_received, CHAT},
  tick::handle_tick,
};
use super::IncomingEvent;
use crate::events::handlers::{block_future, spawn_future};

pub fn handle_incoming_event(event: IncomingEvent) {
  match event {
    IncomingEvent::Tick => {
      handle_tick();
    }

    IncomingEvent::ChatReceived(message, msg_type) => {
      spawn_future(async move {
        handle_chat_received(message, msg_type).await;
      });
    }

    IncomingEvent::InputPress(key) => {
      CHAT.with(move |chat| {
        let mut chat = chat.borrow_mut();
        block_future(async move {
          chat.handle_key_press(key).await;
        });
      });
    }

    IncomingEvent::InputDown(key, repeat) => {
      CHAT.with(move |chat| {
        let mut chat = chat.borrow_mut();
        block_future(async move {
          chat.handle_key_down(key, repeat).await;
        });
      });
    }

    IncomingEvent::InputUp(key) => {
      CHAT.with(move |chat| {
        let mut chat = chat.borrow_mut();
        block_future(async move {
          chat.handle_key_up(key).await;
        });
      });
    }
  }
}
