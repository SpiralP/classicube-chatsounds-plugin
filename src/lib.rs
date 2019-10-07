mod plugin;
mod printer;

use classicube::IGameComponent;
use std::{os::raw::c_int, ptr};

extern "C" fn free() {
  plugin::unload();
}

extern "C" fn on_new_map_loaded() {
  plugin::load();
}

#[no_mangle]
pub static Plugin_ApiVersion: c_int = 1;

#[no_mangle]
pub static mut Plugin_Component: IGameComponent = IGameComponent {
  Init: None,
  Free: Some(free),
  Reset: Some(free),
  OnNewMap: None,
  OnNewMapLoaded: Some(on_new_map_loaded),
  next: ptr::null_mut(),
};
