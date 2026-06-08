mod outgoing_events;
mod types;

use std::{
    cell::{Cell, RefCell},
    os::raw::c_int,
};

use classicube_helpers::{
    chat::ProtocolMessageHook,
    events::{
        chat::{ChatReceivedEvent, ChatReceivedEventHandler},
        input,
        window::FocusChangedEventHandler,
    },
    tick::TickEventHandler,
};
use classicube_sys::{
    Chat_Add, Chat_AddOf, Event_RaiseInput, Event_RaiseInt, InputDevice, InputEvents,
    MsgType_MSG_TYPE_NORMAL, OwnedString, Server, WindowInfo,
};
use crossbeam_channel::{Receiver, Sender, unbounded};
pub use outgoing_events::*;
use parking_lot::Mutex;
use tracing::debug;

pub use self::types::*;
use crate::{
    helpers::{is_global_cs_message, is_global_csent_message, is_global_cspos_message},
    modules::Module,
};

thread_local!(
    static DEVICE: Cell<Option<*mut InputDevice>> = Cell::default();
);

thread_local!(
    static EVENT_HANDLER_MODULE: Cell<Option<*mut EventHandlerModule>> = const { Cell::new(None) };
);

// Persistent across load/unload cycles: Protocol.Handlers is wiped by the
// Protocol component's OnReset (which fires before this plugin's Reset), so
// we keep the hook handle alive and call reinstall() each cycle rather than
// dropping and re-installing (which would trip the IN_CHAIN veto and silently
// skip the re-push).
thread_local!(
    static MESSAGE_HOOK: RefCell<Option<ProtocolMessageHook>> = const { RefCell::new(None) };
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

impl Module for EventHandlerModule {
    #[expect(
        clippy::too_many_lines,
        reason = "single-function event wiring; no natural split"
    )]
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
            MESSAGE_HOOK.with_borrow_mut(|hook| {
                if let Some(hook) = hook {
                    // Reconnect: Protocol component's OnReset fires before ours
                    // and zeros Protocol.Handlers; re-push our trampoline onto
                    // the restored stock handler.
                    hook.reinstall();
                } else {
                    *hook = ProtocolMessageHook::install(move |text: &str| {
                        let Some(ptr) = EVENT_HANDLER_MODULE.get() else {
                            return false;
                        };
                        let module = unsafe { &mut *ptr };

                        if !module.simulating {
                            module.handle_incoming_event(&IncomingEvent::ChatReceived(
                                text.to_string(),
                                MsgType_MSG_TYPE_NORMAL,
                            ));
                            module.handle_outgoing_events();
                        }

                        if let Some(text) = is_global_cs_message(text) {
                            debug!(?text, "hide global cs message");
                            true
                        } else if let Some((text, pos)) = is_global_cspos_message(text) {
                            debug!(?text, ?pos, "hide global cspos message");
                            true
                        } else if let Some((text, id)) = is_global_csent_message(text) {
                            debug!(?text, ?id, "hide global csent message");
                            true
                        } else {
                            false
                        }
                    });
                }
            });
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
        // The message hook (MESSAGE_HOOK thread-local) is process-resident and
        // not dropped here; EVENT_HANDLER_MODULE.set(None) below makes the
        // trampoline inert until the next load() calls reinstall().

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
