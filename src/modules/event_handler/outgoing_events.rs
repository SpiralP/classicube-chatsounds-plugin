use classicube_sys::{InputButtons, MsgType};

use super::InputDeviceSend;
use crate::modules::event_handler::{OutgoingEvent, OUTGOING_SENDER};

pub fn new_outgoing_event(event: OutgoingEvent) {
    let mut outgoing_sender = OUTGOING_SENDER.lock();
    if let Some(sender) = outgoing_sender.as_mut() {
        sender.send(event).unwrap();
    }
}

pub fn chat_add_of<S: Into<String>>(text: S, msg_type: MsgType) {
    new_outgoing_event(OutgoingEvent::ChatAddOf(text.into(), msg_type));
}

pub fn chat_add<S: Into<String>>(text: S) {
    new_outgoing_event(OutgoingEvent::ChatAdd(text.into()));
}

pub fn simulate_key(key: InputButtons, device: InputDeviceSend) {
    new_outgoing_event(OutgoingEvent::InputDown(key, false, device));
    new_outgoing_event(OutgoingEvent::InputUp(key, false, device));
}

pub fn simulate_char(chr: char) {
    new_outgoing_event(OutgoingEvent::InputPress(chr));
}
