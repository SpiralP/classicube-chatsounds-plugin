mod chat;

use self::chat::Chat;
use crate::modules::{
  event_handler::{IncomingEvent, IncomingEventListener},
  EventHandlerModule, FuturesModule, Module, OptionModule,
};
use chatsounds::Chatsounds;
use futures::{
  channel::mpsc::{unbounded, UnboundedSender},
  lock::Mutex,
  prelude::*,
};
use std::{cell::RefCell, rc::Rc, sync::Arc};

pub struct AutocompleteModule {
  option_module: Rc<RefCell<OptionModule>>,
  chatsounds: Arc<Mutex<Chatsounds>>,
  event_handler_module: Rc<RefCell<EventHandlerModule>>,
}

impl AutocompleteModule {
  pub fn new(
    option_module: Rc<RefCell<OptionModule>>,
    chatsounds: Arc<Mutex<Chatsounds>>,
    event_handler_module: Rc<RefCell<EventHandlerModule>>,
  ) -> Self {
    Self {
      option_module,
      chatsounds,
      event_handler_module,
    }
  }
}

impl Module for AutocompleteModule {
  fn load(&mut self) {
    let autocomplete_event_listener =
      AutocompleteEventListener::new(self.option_module.clone(), self.chatsounds.clone());

    self
      .event_handler_module
      .borrow_mut()
      .register_listener(autocomplete_event_listener);
  }

  fn unload(&mut self) {}
}

pub struct AutocompleteEventListener {
  sender: UnboundedSender<IncomingEvent>,
}

impl AutocompleteEventListener {
  pub fn new(option_module: Rc<RefCell<OptionModule>>, chatsounds: Arc<Mutex<Chatsounds>>) -> Self {
    let (sender, mut receiver) = unbounded();

    let mut chat = Chat::new(option_module, chatsounds);

    FuturesModule::spawn_future(async move {
      while let Some(event) = receiver.next().await {
        match event {
          IncomingEvent::InputPress(key) => {
            chat.handle_key_press(key).await;
          }

          IncomingEvent::InputDown(key, repeat) => {
            chat.handle_key_down(key, repeat).await;
          }

          IncomingEvent::InputUp(key) => {
            chat.handle_key_up(key).await;
          }

          _ => {}
        }
      }
    });

    Self { sender }
  }
}

impl IncomingEventListener for AutocompleteEventListener {
  fn handle_incoming_event(&mut self, event: &IncomingEvent) {
    match event {
      IncomingEvent::InputPress(_) | IncomingEvent::InputDown(_, _) | IncomingEvent::InputUp(_) => {
        // TODO somehow block here on tab key_down

        // send and process in the same order
        FuturesModule::block_future(self.sender.send(event.clone())).unwrap();
      }

      _ => {}
    }
  }
}
