mod chat;
mod tick;

use self::{
  chat::{handle_chat_received, CHAT},
  tick::handle_tick,
};
use super::IncomingEvent;

pub fn handle_incoming_event(event: IncomingEvent) {
  match event {
    IncomingEvent::Tick => {
      handle_tick();
    }

    IncomingEvent::ChatReceived(message, msg_type) => {
      handle_chat_received(message, msg_type);
    }

    IncomingEvent::InputPress(key) => {
      CHAT.with(move |chat| {
        let mut chat = chat.borrow_mut();
        chat.handle_key_press(key);
      });
    }

    IncomingEvent::InputDown(key, repeat) => {
      CHAT.with(move |chat| {
        let mut chat = chat.borrow_mut();
        chat.handle_key_down(key, repeat);
      });
    }

    IncomingEvent::InputUp(key) => {
      CHAT.with(move |chat| {
        let mut chat = chat.borrow_mut();
        chat.handle_key_up(key);
      });
    }
  }
}
