mod chat;
mod chatsounds;
mod command;
mod events;
mod option;
mod plugin;
mod printer;
mod thread;

use classicube::IGameComponent;
use std::{cell::Cell, os::raw::c_int, ptr};

thread_local! {
  static LOADED: Cell<bool> = Cell::new(false);
}

extern "C" fn on_new_map_loaded() {
  let loaded = LOADED.with(|a| a.get());

  if !loaded {
    plugin::load();
  }

  LOADED.with(|a| a.set(true));
}

extern "C" fn free() {
  let loaded = LOADED.with(|a| a.get());

  if loaded {
    plugin::unload();
  }

  LOADED.with(|a| a.set(false));
}

#[no_mangle]
pub static Plugin_ApiVersion: c_int = 1;

#[no_mangle]
pub static mut Plugin_Component: IGameComponent = IGameComponent {
  /* Called when the game is being loaded. */
  Init: None,
  /* Called when the component is being freed. (e.g. due to game being closed) */
  Free: Some(free),
  /* Called to reset the component's state. (e.g. reconnecting to server) */
  Reset: None,
  /* Called to update the component's state when the user begins loading a new map. */
  OnNewMap: None,
  /* Called to update the component's state when the user has finished loading a new map. */
  OnNewMapLoaded: Some(on_new_map_loaded),
  /* Next component in linked list of components. */
  next: ptr::null_mut(),
};
