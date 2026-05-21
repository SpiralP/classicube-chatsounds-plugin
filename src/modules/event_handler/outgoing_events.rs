use classicube_sys::{Chat_Add, Chat_AddOf, InputButtons, MsgType, OwnedString};

use crate::modules::event_handler::{OUTGOING_SENDER, OutgoingEvent};

pub fn new_outgoing_event(event: OutgoingEvent) {
    let mut outgoing_sender = OUTGOING_SENDER.lock();
    if let Some(sender) = outgoing_sender.as_mut() {
        let _ = sender.send(event);
    }
}

pub fn chat_add_of<S: Into<String>>(text: S, msg_type: MsgType) {
    let text = text.into();
    let mut outgoing_sender = OUTGOING_SENDER.lock();
    if let Some(sender) = outgoing_sender.as_mut() {
        let _ = sender.send(OutgoingEvent::ChatAddOf(text, msg_type));
    } else {
        // Plugin is unloaded (between Free/Init): the queue is gone and no
        // listener would drain it. The only callers that can reach this in
        // the unloaded state are ClassiCube's main-thread callbacks (e.g.
        // the never-unregistered chat command), so calling Chat_AddOf
        // directly is sound.
        let owned_string = OwnedString::new(text);
        unsafe {
            Chat_AddOf(owned_string.as_cc_string(), msg_type.try_into().unwrap());
        }
    }
}

pub fn chat_add<S: Into<String>>(text: S) {
    let text = text.into();
    let mut outgoing_sender = OUTGOING_SENDER.lock();
    if let Some(sender) = outgoing_sender.as_mut() {
        let _ = sender.send(OutgoingEvent::ChatAdd(text));
    } else {
        let owned_string = OwnedString::new(text);
        unsafe {
            Chat_Add(owned_string.as_cc_string());
        }
    }
}

pub fn simulate_key(key: InputButtons) {
    new_outgoing_event(OutgoingEvent::InputDown(key, false));
    new_outgoing_event(OutgoingEvent::InputUp(key, false));
}

pub fn simulate_char(chr: char) {
    new_outgoing_event(OutgoingEvent::InputPress(chr));
}
