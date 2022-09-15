pub mod app_name;
pub mod autocomplete;
pub mod chatsounds;
pub mod command;
pub mod event_handler;
pub mod futures;
pub mod option;

use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};

use ::futures::lock::Mutex as FutureMutex;
use classicube_helpers::{entities::Entities, tab_list::TabList};

pub use self::{
    app_name::AppNameModule, autocomplete::AutocompleteModule, chatsounds::ChatsoundsModule,
    command::CommandModule, event_handler::EventHandlerModule, futures::FuturesModule,
    option::OptionModule,
};
use crate::printer::PrinterEventListener;

pub trait Module {
    fn load(&mut self);
    fn unload(&mut self);
}

pub type SyncShared<T> = Rc<RefCell<T>>;
pub type ThreadShared<T> = Arc<Mutex<T>>;
pub type FutureShared<T> = Arc<FutureMutex<T>>;

thread_local!(
    static MODULES: RefCell<Vec<SyncShared<dyn Module>>> = RefCell::new(Vec::new());
);

pub fn load() {
    MODULES.with(|ref_cell| {
        let mut modules = ref_cell.borrow_mut();

        // TODO maybe give eachother Weak?

        let entities = Rc::new(RefCell::new(Entities::new()));
        let tab_list = Rc::new(RefCell::new(TabList::new()));

        let option_module = Rc::new(RefCell::new(OptionModule::new()));
        modules.push(option_module.clone());

        let event_handler_module = Rc::new(RefCell::new(EventHandlerModule::new()));
        event_handler_module
            .borrow_mut()
            .register_listener(PrinterEventListener {});
        modules.push(event_handler_module.clone());

        let app_name_module = Rc::new(RefCell::new(AppNameModule::new()));
        modules.push(app_name_module);

        let futures_module = Rc::new(RefCell::new(FuturesModule::new()));
        modules.push(futures_module);

        let chatsounds_module = Rc::new(RefCell::new(ChatsoundsModule::new(
            option_module.clone(),
            entities,
            event_handler_module.clone(),
            tab_list,
        )));
        modules.push(chatsounds_module.clone());

        let command_module = Rc::new(RefCell::new(CommandModule::new(
            option_module.clone(),
            event_handler_module.clone(),
            chatsounds_module.borrow_mut().chatsounds.clone(),
        )));
        modules.push(command_module);

        let autocomplete_module = Rc::new(RefCell::new(AutocompleteModule::new(
            option_module,
            chatsounds_module.borrow_mut().chatsounds.clone(),
            event_handler_module,
        )));
        modules.push(autocomplete_module);

        for module in modules.iter_mut() {
            let mut module = module.borrow_mut();
            module.load();
        }
    });
}

pub fn unload() {
    MODULES.with(|ref_cell| {
        let mut modules = ref_cell.borrow_mut();

        // TODO using Rc will keep these alive in other places on unload!

        // unload in reverse order
        for module in modules.drain(..).rev() {
            let mut module = module.borrow_mut();
            module.unload();
        }
    });
}
