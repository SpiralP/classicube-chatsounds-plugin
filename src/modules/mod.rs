pub mod app_name;
pub mod autocomplete;
pub mod chatsounds;
pub mod command;
pub mod entities;
pub mod event_handler;
pub mod futures;
pub mod option;
pub mod tab_list;

use std::{cell::RefCell, rc::Rc};

pub use self::{
  app_name::AppNameModule, autocomplete::AutocompleteModule, chatsounds::ChatsoundsModule,
  command::CommandModule, entities::EntitiesModule, event_handler::EventHandlerModule,
  futures::FuturesModule, option::OptionModule, tab_list::TabListModule,
};

pub trait Module {
  fn load(&mut self);
  fn unload(&mut self);
}

thread_local! {
  static MODULES: RefCell<Vec<Rc<RefCell<dyn Module>>>> = RefCell::new(Vec::new());
}

pub fn load() {
  MODULES.with(|ref_cell| {
    let mut modules = ref_cell.borrow_mut();

    // TODO maybe give eachother Weak?

    let entities_module = Rc::new(RefCell::new(EntitiesModule::new()));
    modules.push(entities_module.clone());

    let tab_list_module = Rc::new(RefCell::new(TabListModule::new()));
    modules.push(tab_list_module.clone());

    let option_module = Rc::new(RefCell::new(OptionModule::new()));
    modules.push(option_module.clone());

    let event_handler_module = Rc::new(RefCell::new(EventHandlerModule::new()));
    modules.push(event_handler_module.clone());

    let app_name_module = Rc::new(RefCell::new(AppNameModule::new()));
    modules.push(app_name_module);

    let futures_module = Rc::new(RefCell::new(FuturesModule::new()));
    modules.push(futures_module.clone());

    let chatsounds_module = Rc::new(RefCell::new(ChatsoundsModule::new(
      option_module.clone(),
      futures_module.clone(),
      entities_module,
      event_handler_module.clone(),
      tab_list_module,
    )));
    modules.push(chatsounds_module.clone());

    let command_module = Rc::new(RefCell::new(CommandModule::new(
      option_module.clone(),
      event_handler_module.clone(),
      futures_module,
      chatsounds_module.clone(),
    )));
    modules.push(command_module);

    let autocomplete_module = Rc::new(RefCell::new(AutocompleteModule::new(
      option_module,
      chatsounds_module,
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
