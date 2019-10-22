mod chat;
mod tick;

use self::{
  chat::{handle_chat_received, CHAT},
  tick::handle_tick,
};
use super::IncomingEvent;
use crate::events::handlers::spawn_future;

pub fn handle_incoming_event(event: IncomingEvent) {
  spawn_future(async move {
    match event {
      IncomingEvent::Tick => {
        handle_tick();
      }

      IncomingEvent::ChatReceived(message, msg_type) => {
        handle_chat_received(message, msg_type).await;
      }

      IncomingEvent::InputPress(key) => {
        let mut chat = CHAT.lock().await;
        chat.handle_key_press(key).await;
      }

      IncomingEvent::InputDown(key, repeat) => {
        let mut chat = CHAT.lock().await;
        chat.handle_key_down(key, repeat).await;
      }

      IncomingEvent::InputUp(key) => {
        let mut chat = CHAT.lock().await;
        chat.handle_key_up(key).await;
      }
    }
  });
}
