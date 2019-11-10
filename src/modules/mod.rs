pub mod app_name;
pub mod autocomplete;
pub mod chatsounds;
pub mod command;
pub mod event_handler;
pub mod futures;
pub mod option;
mod shared;

pub use self::{
  app_name::AppNameModule,
  autocomplete::AutocompleteModule,
  chatsounds::ChatsoundsModule,
  command::CommandModule,
  event_handler::EventHandlerModule,
  futures::FuturesModule,
  option::OptionModule,
  shared::{FutureShared, SyncShared, ThreadShared},
};
use crate::printer::PrinterEventListener;
use classicube_helpers::{Entities, TabList};
use std::cell::RefCell;

pub trait Module {
  fn load(&mut self);
  fn unload(&mut self);
}

thread_local! {
  static MODULES: RefCell<Vec<SyncShared<dyn Module>>> = RefCell::new(Vec::new());
}

pub fn load() {
  MODULES.with(|ref_cell| {
    let mut modules = ref_cell.borrow_mut();

    // TODO maybe give eachother Weak?

    let entities = SyncShared::new(Entities::register());
    let tab_list = SyncShared::new(TabList::register());

    let option_module = SyncShared::new(OptionModule::new());
    modules.push(option_module.clone());

    let mut event_handler_module = SyncShared::new(EventHandlerModule::new());
    event_handler_module
      .lock()
      .register_listener(PrinterEventListener {});
    modules.push(event_handler_module.clone());

    let app_name_module = SyncShared::new(AppNameModule::new());
    modules.push(app_name_module);

    let futures_module = SyncShared::new(FuturesModule::new());
    modules.push(futures_module);

    let mut chatsounds_module = SyncShared::new(ChatsoundsModule::new(
      option_module.clone(),
      entities,
      event_handler_module.clone(),
      tab_list,
    ));
    modules.push(chatsounds_module.clone());

    let command_module = SyncShared::new(CommandModule::new(
      option_module.clone(),
      event_handler_module.clone(),
      chatsounds_module.clone(),
    ));
    modules.push(command_module);

    let autocomplete_module = SyncShared::new(AutocompleteModule::new(
      option_module,
      chatsounds_module.lock().chatsounds.clone(),
      event_handler_module,
    ));
    modules.push(autocomplete_module);

    for module in modules.iter_mut() {
      let mut module = module.lock();
      module.load();
    }
  });
}

pub fn unload() {
  MODULES.with(|ref_cell| {
    let mut modules = ref_cell.borrow_mut();

    // TODO using Rc will keep these alive in other places on unload!

    // unload in reverse order
    for mut module in modules.drain(..).rev() {
      let mut module = module.lock();
      module.unload();
    }
  });
}
