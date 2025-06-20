mod helpers;
mod logger;
mod modules;
mod printer;

use std::{os::raw::c_int, ptr};

use classicube_sys::IGameComponent;
use lazy_static::lazy_static;
use parking_lot::Mutex;
use tracing::debug;

lazy_static! {
    static ref LOADED: Mutex<bool> = Mutex::new(false);
}

extern "C" fn init() {
    let mut loaded = LOADED.lock();

    if !*loaded {
        color_backtrace::install_with_settings(
            color_backtrace::Settings::new().verbosity(color_backtrace::Verbosity::Full),
        );

        logger::initialize(true, false);

        debug!("init: modules::load()");
        modules::load();

        *loaded = true;
    }
}

extern "C" fn reset() {
    let mut loaded = LOADED.lock();

    if *loaded {
        debug!("reset: modules::unload()");
        modules::unload();

        *loaded = false;
    }

    if !*loaded {
        debug!("reset: modules::load()");
        modules::load();

        *loaded = true;
    }
}

extern "C" fn free() {
    let mut loaded = LOADED.lock();

    if *loaded {
        debug!("free: modules::unload()");
        modules::unload();

        *loaded = false;
    }
}

#[allow(non_upper_case_globals)]
#[no_mangle]
pub static Plugin_ApiVersion: c_int = 1;

#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut Plugin_Component: IGameComponent = IGameComponent {
    // Called when the game is being loaded.
    Init: Some(init),
    // Called when the component is being freed. (e.g. due to game being closed)
    Free: Some(free),
    // Called to reset the component's state. (e.g. reconnecting to server)
    Reset: Some(reset),
    // Called to update the component's state when the user begins loading a new map.
    OnNewMap: None,
    // Called to update the component's state when the user has finished loading a new map.
    OnNewMapLoaded: None,
    // Next component in linked list of components.
    next: ptr::null_mut(),
};
