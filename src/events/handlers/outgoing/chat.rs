use super::OutgoingEvent;
use crate::events::handlers::outgoing::new_outgoing_event;
use classicube_sys::{Key_, MsgType};

pub fn chat_add_of<S: Into<String>>(text: S, msg_type: MsgType) {
  new_outgoing_event(OutgoingEvent::ChatAddOf(text.into(), msg_type));
}

pub fn chat_add<S: Into<String>>(text: S) {
  new_outgoing_event(OutgoingEvent::ChatAdd(text.into()));
}

pub fn simulate_key(key: Key_) {
  new_outgoing_event(OutgoingEvent::InputDown(key, false));
  new_outgoing_event(OutgoingEvent::InputUp(key));
}

pub fn simulate_char(chr: char) {
  new_outgoing_event(OutgoingEvent::InputPress(chr));
}
