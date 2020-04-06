mod callbacks;
mod outgoing_events;
mod types;

use self::callbacks::{on_chat_received, on_input_down, on_input_press, on_input_up};
pub use self::types::{IncomingEvent, OutgoingEvent};
use crate::modules::Module;
use classicube_helpers::{TickEventListener, TickEventType};
use classicube_sys::{
  ChatEvents, Chat_Add, Chat_AddOf, Event_RaiseInput, Event_RaiseInt, Event_RegisterChat,
  Event_RegisterInput, Event_RegisterInt, Event_UnregisterChat, Event_UnregisterInput,
  Event_UnregisterInt, InputEvents, OwnedString, ScheduledTask, Server,
};
use crossbeam_channel::{unbounded, Receiver, Sender};
use lazy_static::lazy_static;
pub use outgoing_events::*;
use parking_lot::Mutex;
use std::{
  cell::Cell,
  os::raw::{c_int, c_void},
};

// TODO move this file logic to helpers lib so that we don't have 2 layers of event handlers

// hack so that our tick detour can fire tick events
thread_local!(
  static EVENT_HANDLER_MODULE: Cell<Option<*mut EventHandlerModule>> = Cell::new(None);
);

lazy_static! {
  pub static ref OUTGOING_SENDER: Mutex<Option<Sender<OutgoingEvent>>> = Mutex::new(None);
}

pub trait IncomingEventListener {
  // TODO maybe a on_registered fn

  fn handle_incoming_event(&mut self, event: &IncomingEvent);
}

pub struct EventHandlerModule {
  simulating: bool,
  incoming_event_listeners: Vec<Box<dyn IncomingEventListener>>,
  outgoing_event_sender: Option<Sender<OutgoingEvent>>,
  outgoing_event_receiver: Receiver<OutgoingEvent>,
  tick_callback: Option<TickEventListener>,
}

impl EventHandlerModule {
  pub fn new() -> Self {
    let (outgoing_event_sender, outgoing_event_receiver) = unbounded();

    Self {
      simulating: false,
      incoming_event_listeners: Vec::new(),
      outgoing_event_sender: Some(outgoing_event_sender),
      outgoing_event_receiver,
      tick_callback: None,
    }
  }

  pub fn register_listener<L>(&mut self, listener: L)
  where
    L: IncomingEventListener,
    L: 'static,
  {
    self.incoming_event_listeners.push(Box::new(listener));
  }

  pub fn handle_incoming_event(&mut self, event: IncomingEvent) {
    for listener in self.incoming_event_listeners.iter_mut() {
      listener.handle_incoming_event(&event);
    }
  }

  pub fn handle_outgoing_events(&mut self) {
    self.simulating = true;

    for event in self.outgoing_event_receiver.try_iter() {
      self.handle_outgoing_event(event);
    }

    self.simulating = false;
  }

  fn handle_outgoing_event(&self, event: OutgoingEvent) {
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
          Chat_AddOf(owned_string.as_cc_string(), msg_type as _);
        }
      }

      OutgoingEvent::InputPress(chr) => unsafe {
        Event_RaiseInt(&mut InputEvents.Press, c_int::from(chr as u8));
      },

      OutgoingEvent::InputDown(key, repeat) => unsafe {
        Event_RaiseInput(&mut InputEvents.Down, key as _, repeat);
      },

      OutgoingEvent::InputUp(key) => unsafe {
        Event_RaiseInt(&mut InputEvents.Up, key as _);
      },
    }
  }
}

impl Module for EventHandlerModule {
  fn load(&mut self) {
    {
      let mut outgoing_sender = OUTGOING_SENDER.lock();
      *outgoing_sender = self.outgoing_event_sender.take();
    }

    // TODO should this be Pin??
    let ptr: *mut EventHandlerModule = self;
    unsafe {
      Event_RegisterChat(
        &mut ChatEvents.ChatReceived,
        ptr as *mut c_void,
        Some(on_chat_received),
      );

      Event_RegisterInput(
        &mut InputEvents.Down,
        ptr as *mut c_void,
        Some(on_input_down),
      );
      Event_RegisterInt(&mut InputEvents.Up, ptr as *mut c_void, Some(on_input_up));
      Event_RegisterInt(
        &mut InputEvents.Press,
        ptr as *mut c_void,
        Some(on_input_press),
      );
    }

    let mut tick_callback = TickEventListener::register();
    tick_callback.on(TickEventType::Tick, |_event| {
      EVENT_HANDLER_MODULE.with(|maybe_ptr| {
        if let Some(ptr) = maybe_ptr.get() {
          let event_handler = unsafe { &mut *ptr };
          event_handler.handle_incoming_event(IncomingEvent::Tick);
          event_handler.handle_outgoing_events();
        }
      });
    });

    self.tick_callback = Some(tick_callback);

    EVENT_HANDLER_MODULE.with(|cell| {
      cell.set(Some(ptr));
    });
  }

  fn unload(&mut self) {
    EVENT_HANDLER_MODULE.with(|cell| {
      cell.take();
    });

    {
      self.tick_callback.take();
    }

    let ptr: *mut EventHandlerModule = self;
    unsafe {
      Event_UnregisterChat(
        &mut ChatEvents.ChatReceived,
        ptr as *mut c_void,
        Some(on_chat_received),
      );

      Event_UnregisterInput(
        &mut InputEvents.Down,
        ptr as *mut c_void,
        Some(on_input_down),
      );
      Event_UnregisterInt(&mut InputEvents.Up, ptr as *mut c_void, Some(on_input_up));
      Event_UnregisterInt(
        &mut InputEvents.Press,
        ptr as *mut c_void,
        Some(on_input_press),
      );
    }
  }
}
