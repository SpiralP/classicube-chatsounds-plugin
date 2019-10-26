mod chat;

use self::chat::Chat;
use crate::modules::{
  event_handler::{IncomingEvent, IncomingEventListener},
  ChatsoundsModule, EventHandlerModule, Module, OptionModule,
};
use std::{cell::RefCell, rc::Rc};

pub struct AutocompleteModule {}

impl AutocompleteModule {
  pub fn new(
    option_module: Rc<RefCell<OptionModule>>,
    chatsounds_module: Rc<RefCell<ChatsoundsModule>>,
    event_handler_module: Rc<RefCell<EventHandlerModule>>,
  ) -> Self {
    let autocomplete_event_listener =
      AutocompleteEventListener::new(option_module, chatsounds_module);
    event_handler_module
      .borrow_mut()
      .register_listener(autocomplete_event_listener);

    Self {}
  }
}

impl Module for AutocompleteModule {
  fn load(&mut self) {}
  fn unload(&mut self) {}
}

pub struct AutocompleteEventListener {
  chat: Chat,
}

impl AutocompleteEventListener {
  pub fn new(
    option_module: Rc<RefCell<OptionModule>>,
    chatsounds_module: Rc<RefCell<ChatsoundsModule>>,
  ) -> Self {
    Self {
      chat: Chat::new(option_module, chatsounds_module),
    }
  }
}

impl IncomingEventListener for AutocompleteEventListener {
  fn handle_incoming_event(&mut self, event: &IncomingEvent) {
    match event.clone() {
      IncomingEvent::InputPress(key) => {
        self.chat.handle_key_press(key);
      }

      IncomingEvent::InputDown(key, repeat) => {
        self.chat.handle_key_down(key, repeat);
      }

      IncomingEvent::InputUp(key) => {
        self.chat.handle_key_up(key);
      }

      _ => {}
    }
  }
}
