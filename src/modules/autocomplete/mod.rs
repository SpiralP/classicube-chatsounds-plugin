mod chat;

use chatsounds::Chatsounds;
use classicube_helpers::tab_list::{TabList, remove_color};
use futures::{
    channel::mpsc::{UnboundedSender, unbounded},
    prelude::*,
};

use self::chat::Chat;
use crate::modules::{
    EventHandlerModule, FutureShared, FuturesModule, Module, OptionModule, SyncShared,
    ThreadShared,
    event_handler::{IncomingEvent, IncomingEventListener},
};

pub struct AutocompleteModule {
    option_module: SyncShared<OptionModule>,
    chatsounds: FutureShared<Option<Chatsounds>>,
    event_handler_module: SyncShared<EventHandlerModule>,
    tab_list: SyncShared<TabList>,
}

impl AutocompleteModule {
    pub fn new(
        option_module: SyncShared<OptionModule>,
        chatsounds: FutureShared<Option<Chatsounds>>,
        event_handler_module: SyncShared<EventHandlerModule>,
        tab_list: SyncShared<TabList>,
    ) -> Self {
        Self {
            option_module,
            chatsounds,
            event_handler_module,
            tab_list,
        }
    }
}

impl Module for AutocompleteModule {
    fn load(&mut self) {
        let autocomplete_event_listener = AutocompleteEventListener::new(
            &self.option_module,
            self.chatsounds.clone(),
            self.tab_list.clone(),
        );

        self.event_handler_module
            .borrow_mut()
            .register_listener(autocomplete_event_listener);
    }

    fn unload(&mut self) {}
}

pub struct AutocompleteEventListener {
    sender: UnboundedSender<IncomingEvent>,
    tab_list: SyncShared<TabList>,
    player_names: ThreadShared<Vec<String>>,
}

impl AutocompleteEventListener {
    pub fn new(
        option_module: &SyncShared<OptionModule>,
        chatsounds: FutureShared<Option<Chatsounds>>,
        tab_list: SyncShared<TabList>,
    ) -> Self {
        let player_names: ThreadShared<Vec<String>> = ThreadShared::default();

        let (sender, mut receiver) = unbounded();

        let mut chat = Chat::new(option_module, chatsounds, player_names.clone());

        FuturesModule::spawn_future(async move {
            while let Some(event) = receiver.next().await {
                if !OptionModule::autocomplete() {
                    continue;
                }

                match event {
                    IncomingEvent::InputPress(key) => {
                        chat.handle_key_press(key).await;
                    }

                    IncomingEvent::InputDown(key, repeat) => {
                        chat.handle_key_down(key, repeat).await;
                    }

                    IncomingEvent::InputUp(key, repeating) => {
                        chat.handle_key_up(key, repeating);
                    }

                    _ => {}
                }
            }
        });

        Self {
            sender,
            tab_list,
            player_names,
        }
    }

    fn refresh_player_names(&self) {
        let names: Vec<String> = self
            .tab_list
            .borrow()
            .get_all()
            .iter()
            .filter_map(|(_id, weak)| weak.upgrade())
            .map(|e| remove_color(e.get_real_name()).trim().to_string())
            .filter(|n| !n.is_empty())
            .collect();
        *self.player_names.lock().unwrap() = names;
    }
}

impl IncomingEventListener for AutocompleteEventListener {
    fn handle_incoming_event(&mut self, event: &IncomingEvent) {
        match event {
            // InputDown always fires before its paired InputPress, so one
            // refresh here covers both the char-typed and key-action paths.
            IncomingEvent::InputDown(..) => {
                self.refresh_player_names();
                FuturesModule::block_future(self.sender.send(event.clone())).unwrap();
            }

            IncomingEvent::InputPress(_) | IncomingEvent::InputUp(..) => {
                // TODO somehow block here on tab key_down

                // send and process in the same order
                FuturesModule::block_future(self.sender.send(event.clone())).unwrap();
            }

            _ => {}
        }
    }
}
