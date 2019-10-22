use crate::{
  app_name, command, entities, events,
  events::{block_future, spawn_future},
  option, tablist,
};

pub fn load() {
  events::load();
  command::load();
  option::load();
  entities::load();
  tablist::load();
  app_name::load();

  spawn_future(async {
    crate::chatsounds::load().await;
  });
}

pub fn unload() {
  block_future(async {
    crate::chatsounds::unload().await;
  });

  app_name::unload();
  tablist::unload();
  entities::unload();
  option::unload();
  command::unload();
  events::unload();
}

// pub fn reset() {
//   ENTITIES.with(|entities| {
//     let mut entities = entities.borrow_mut();
//     entities.clear();
//   });
// }

// pub fn on_new_map() {
//   ENTITIES.with(|entities| {
//     let mut entities = entities.borrow_mut();
//     entities.clear();
//   });
// }

// pub fn on_new_map_loaded() {
//   ENTITIES.with(|entities| {
//     let mut entities = entities.borrow_mut();
//     *entities = get_all_entities();
//   });
// }
