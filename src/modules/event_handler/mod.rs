mod outgoing_events;
mod types;

use std::{cell::Cell, os::raw::c_int};

use classicube_helpers::{
    events::{
        chat::{ChatReceivedEvent, ChatReceivedEventHandler},
        input,
    },
    tick::TickEventHandler,
};
use classicube_sys::{
    Chat_Add, Chat_AddOf, Event_RaiseInput, Event_RaiseInt, InputDevice, InputEvents, OwnedString,
};
use crossbeam_channel::{unbounded, Receiver, Sender};
use lazy_static::lazy_static;
pub use outgoing_events::*;
use parking_lot::Mutex;

pub use self::types::*;
use crate::modules::Module;

thread_local!(
    static DEVICE: Cell<Option<*mut InputDevice>> = Default::default();
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
    chat_received: ChatReceivedEventHandler,
    input_down: input::Down2EventHandler,
    input_press: input::PressEventHandler,
    input_up: input::Up2EventHandler,
    tick_callback: TickEventHandler,
}

impl EventHandlerModule {
    pub fn new() -> Self {
        let (outgoing_event_sender, outgoing_event_receiver) = unbounded();

        Self {
            simulating: false,
            incoming_event_listeners: Vec::new(),
            outgoing_event_sender: Some(outgoing_event_sender),
            outgoing_event_receiver,
            chat_received: ChatReceivedEventHandler::new(),
            input_down: input::Down2EventHandler::new(),
            input_press: input::PressEventHandler::new(),
            input_up: input::Up2EventHandler::new(),
            tick_callback: TickEventHandler::new(),
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
                Event_RaiseInt(&raw mut InputEvents.Press, c_int::from(chr as u8));
            },

            OutgoingEvent::InputDown(key, repeating) => {
                if let Some(device) = DEVICE.get() {
                    unsafe {
                        Event_RaiseInput(
                            #[allow(static_mut_refs)]
                            &mut InputEvents.Down2,
                            key as _,
                            u8::from(repeating),
                            device,
                        );
                    }
                }
            }

            OutgoingEvent::InputUp(key, repeating) => {
                if let Some(device) = DEVICE.get() {
                    unsafe {
                        Event_RaiseInput(
                            #[allow(static_mut_refs)]
                            &mut InputEvents.Up2,
                            key as _,
                            u8::from(repeating),
                            device,
                        );
                    }
                }
            }
        }
    }
}

impl Module for EventHandlerModule {
    fn load(&mut self) {
        {
            let mut outgoing_sender = OUTGOING_SENDER.lock();
            *outgoing_sender = self.outgoing_event_sender.take();
        }

        // TODO maybe use UnsafeCell here so we're a little safer?
        // or describe why we can use pointers here
        let ptr: *mut EventHandlerModule = self;

        self.chat_received.on(
            move |ChatReceivedEvent {
                      message,
                      message_type,
                  }| {
                let module = unsafe { &mut *ptr };

                if module.simulating {
                    return;
                }

                module.handle_incoming_event(IncomingEvent::ChatReceived(
                    message.to_string(),
                    *message_type,
                ));
                module.handle_outgoing_events();
            },
        );

        self.input_down.on(
            move |input::Down2Event {
                      key,
                      repeating,
                      device,
                  }| {
                let module = unsafe { &mut *ptr };

                if module.simulating {
                    return;
                }

                if DEVICE.get().is_none() && !device.is_null() {
                    DEVICE.set(Some(*device));
                }
                module.handle_incoming_event(IncomingEvent::InputDown(*key, *repeating));
                module.handle_outgoing_events();
            },
        );

        self.input_press.on(move |input::PressEvent { key }| {
            let module = unsafe { &mut *ptr };

            if module.simulating {
                return;
            }

            module.handle_incoming_event(IncomingEvent::InputPress(*key));
            module.handle_outgoing_events();
        });

        self.input_up
            .on(move |input::Up2Event { key, repeating, .. }| {
                let module = unsafe { &mut *ptr };

                if module.simulating {
                    return;
                }

                module.handle_incoming_event(IncomingEvent::InputUp(*key, *repeating));
                module.handle_outgoing_events();
            });

        self.tick_callback.on(move |_event| {
            let module = unsafe { &mut *ptr };

            module.handle_incoming_event(IncomingEvent::Tick);
            module.handle_outgoing_events();
        });
    }

    fn unload(&mut self) {}
}
