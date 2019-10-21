mod chat;

pub use self::chat::*;
use super::{OutgoingEvent, OUTGOING_RECEIVER, OUTGOING_SENDER};
use crate::events::SIMULATING;
use classicube_sys::{
  Chat_Add, Chat_AddOf, Event_RaiseInput, Event_RaiseInt, InputEvents, OwnedString,
};
use std::os::raw::c_int;

pub fn new_outgoing_event(event: OutgoingEvent) {
  let mut outgoing_sender = OUTGOING_SENDER.lock();
  if let Some(sender) = outgoing_sender.as_mut() {
    sender.send(event).unwrap();
  }
}

pub fn handle_outgoing_events() {
  SIMULATING.with(|simulating| {
    simulating.set(true);
  });

  OUTGOING_RECEIVER.with(|ref_cell| {
    let mut maybe_receiver = ref_cell.borrow_mut();

    if let Some(receiver) = maybe_receiver.as_mut() {
      for event in receiver.try_iter() {
        handle_outgoing_event(event);
      }
    }
  });

  SIMULATING.with(|simulating| {
    simulating.set(false);
  });
}

fn handle_outgoing_event(event: OutgoingEvent) {
  match event {
    OutgoingEvent::ChatAdd(text) => {
      let owned_string = OwnedString::new(text);

      unsafe {
        Chat_Add(owned_string.as_cc_string());
      }
    }

    OutgoingEvent::ChatAddOf(msg, msg_type) => {
      let owned_string = OwnedString::new(msg);

      unsafe {
        Chat_AddOf(owned_string.as_cc_string(), msg_type);
      }
    }

    OutgoingEvent::InputPress(chr) => unsafe {
      Event_RaiseInt(&mut InputEvents.Press, c_int::from(chr as u8));
    },

    OutgoingEvent::InputDown(key, repeat) => unsafe {
      Event_RaiseInput(&mut InputEvents.Down, key, repeat);
    },

    OutgoingEvent::InputUp(key) => unsafe {
      Event_RaiseInt(&mut InputEvents.Up, key);
    },
  }
}
