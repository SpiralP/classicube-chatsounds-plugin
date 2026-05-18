mod outgoing_events;
mod types;

use std::{cell::Cell, os::raw::c_int, slice};

use classicube_helpers::{
    events::{
        chat::{ChatReceivedEvent, ChatReceivedEventHandler},
        input,
        window::FocusChangedEventHandler,
    },
    tick::TickEventHandler,
};
use classicube_sys::{
    Chat_Add, Chat_AddOf, Event_RaiseInput, Event_RaiseInt, InputDevice, InputEvents,
    MsgType_MSG_TYPE_NORMAL, Net_Handler, OwnedString, Protocol, Server, UNSAFE_GetString,
    WindowInfo, OPCODE__OPCODE_MESSAGE,
};
use crossbeam_channel::{unbounded, Receiver, Sender};
pub use outgoing_events::*;
use parking_lot::Mutex;
use tracing::debug;

pub use self::types::*;
use crate::{
    helpers::{is_global_cs_message, is_global_csent_message, is_global_cspos_message},
    is_plugin_active,
    modules::Module,
};

thread_local!(
    static DEVICE: Cell<Option<*mut InputDevice>> = Cell::default();
);

thread_local!(
    static EVENT_HANDLER_MODULE: Cell<Option<*mut EventHandlerModule>> = const { Cell::new(None) };
);

// Cell because Net_Handler (Option<unsafe extern "C" fn(...)>) is Copy.
// Semantics: None == our hook is not installed; Some(prior) == installed,
// `prior` is what Protocol.Handlers[OPCODE_MESSAGE] held before we patched.
thread_local!(
    static OLD_MESSAGE_HANDLER: Cell<Net_Handler> = const { Cell::new(None) };
);

pub static OUTGOING_SENDER: Mutex<Option<Sender<OutgoingEvent>>> = Mutex::new(None);

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
    focus_changed_callback: FocusChangedEventHandler,
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
            focus_changed_callback: FocusChangedEventHandler::new(),
        }
    }

    pub fn register_listener<L>(&mut self, listener: L)
    where
        L: IncomingEventListener,
        L: 'static,
    {
        self.incoming_event_listeners.push(Box::new(listener));
    }

    pub fn handle_incoming_event(&mut self, event: &IncomingEvent) {
        for listener in &mut self.incoming_event_listeners {
            listener.handle_incoming_event(event);
        }
    }

    pub fn handle_outgoing_events(&mut self) {
        self.simulating = true;

        for event in self.outgoing_event_receiver.try_iter() {
            Self::handle_outgoing_event(event);
        }

        self.simulating = false;
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
                    Chat_AddOf(owned_string.as_cc_string(), msg_type.try_into().unwrap());
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
                            key.try_into().unwrap(),
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
                            key.try_into().unwrap(),
                            u8::from(repeating),
                            device,
                        );
                    }
                }
            }
        }
    }
}

extern "C" fn message_handler(data: *mut u8) {
    if is_plugin_active() {
        use classicube_sys::MsgType;

        let bytes = unsafe { slice::from_raw_parts(data, 65) };
        let message_type = MsgType::from(bytes[0]);

        if message_type == MsgType_MSG_TYPE_NORMAL {
            if let Some(ptr) = EVENT_HANDLER_MODULE.get() {
                let text = unsafe { UNSAFE_GetString(&bytes[1..]) }.to_string();
                let module = unsafe { &mut *ptr };

                if !module.simulating {
                    module.handle_incoming_event(&IncomingEvent::ChatReceived(
                        text.clone(),
                        message_type,
                    ));
                    module.handle_outgoing_events();
                }

                if let Some(text) = is_global_cs_message(&text) {
                    debug!(?text, "hide global cs message");
                    return;
                } else if let Some((text, pos)) = is_global_cspos_message(&text) {
                    debug!(?text, ?pos, "hide global cspos message");
                    return;
                } else if let Some((text, id)) = is_global_csent_message(&text) {
                    debug!(?text, ?id, "hide global csent message");
                    return;
                }
            }
        }
    }

    OLD_MESSAGE_HANDLER.with(|cell| {
        if let Some(f) = cell.get() {
            unsafe {
                f(data);
            }
        }
    });
}

pub fn install_message_handler() {
    OLD_MESSAGE_HANDLER.with(|cell| {
        if cell.get().is_some() {
            return; // already installed; uninstall must be called first
        }
        let prior = unsafe { Protocol.Handlers[OPCODE__OPCODE_MESSAGE as usize] };
        unsafe {
            Protocol.Handlers[OPCODE__OPCODE_MESSAGE as usize] = Some(message_handler);
        }
        cell.set(prior);
    });
}

pub fn uninstall_message_handler() {
    OLD_MESSAGE_HANDLER.with(|cell| {
        let prior = cell.get();
        if prior.is_some() {
            // Restore Protocol.Handlers before clearing the cell so that any
            // in-flight invocation of message_handler still finds the prior
            // pointer in OLD_MESSAGE_HANDLER on its fall-through path.
            unsafe {
                Protocol.Handlers[OPCODE__OPCODE_MESSAGE as usize] = prior;
            }
            cell.set(None);
        }
    });
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

        EVENT_HANDLER_MODULE.set(Some(ptr));

        if unsafe { Server.IsSinglePlayer } == 0 {
            install_message_handler();
        } else {
            self.chat_received.on(
                move |ChatReceivedEvent {
                          message,
                          message_type,
                      }| {
                    let module = unsafe { &mut *ptr };

                    if module.simulating {
                        return;
                    }

                    module.handle_incoming_event(&IncomingEvent::ChatReceived(
                        message.clone(),
                        *message_type,
                    ));
                    module.handle_outgoing_events();
                },
            );
        }

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
                module.handle_incoming_event(&IncomingEvent::InputDown(*key, *repeating));
                module.handle_outgoing_events();
            },
        );

        self.input_press.on(move |input::PressEvent { key }| {
            let module = unsafe { &mut *ptr };

            if module.simulating {
                return;
            }

            module.handle_incoming_event(&IncomingEvent::InputPress(*key));
            module.handle_outgoing_events();
        });

        self.input_up
            .on(move |input::Up2Event { key, repeating, .. }| {
                let module = unsafe { &mut *ptr };

                if module.simulating {
                    return;
                }

                module.handle_incoming_event(&IncomingEvent::InputUp(*key, *repeating));
                module.handle_outgoing_events();
            });

        self.tick_callback.on(move |_event| {
            let module = unsafe { &mut *ptr };

            module.handle_incoming_event(&IncomingEvent::Tick);
            module.handle_outgoing_events();
        });

        self.focus_changed_callback.on(move |_event| {
            let module = unsafe { &mut *ptr };

            let focused = unsafe { WindowInfo.Focused } != 0;
            module.handle_incoming_event(&IncomingEvent::FocusChanged(focused));
            module.handle_outgoing_events();
        });
    }

    fn unload(&mut self) {
        // Remove our message_handler from Protocol.Handlers before tearing
        // down per-load state, so it can't fire during the gap and read
        // dangling pointers.
        uninstall_message_handler();

        // Drop the sender before EventHandlerModule (and thus its Receiver)
        // is dropped, so any outgoing_events::new_outgoing_event during the
        // gap sees None instead of a closed channel.
        {
            let mut outgoing_sender = OUTGOING_SENDER.lock();
            *outgoing_sender = None;
        }

        EVENT_HANDLER_MODULE.set(None);
        DEVICE.set(None);

        // classicube-helpers event fields (chat_received, input_*,
        // tick_callback, focus_changed_callback) unregister via Drop when
        // self is dropped — no manual work needed.
    }
}
