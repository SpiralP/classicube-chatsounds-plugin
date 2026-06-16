#![warn(clippy::pedantic)]

mod helpers;
mod logger;
mod modules;
mod printer;

use std::{
    os::raw::c_int,
    ptr,
    sync::atomic::{AtomicBool, Ordering},
};

use classicube_sys::IGameComponent;
use tracing::debug;

// Single source of truth for "modules loaded / callbacks may dereference
// per-load state". ClassiCube invokes Init/Free/Reset from the main thread,
// so a lock-free atomic is sufficient — no Mutex needed.
pub static PLUGIN_ACTIVE: AtomicBool = AtomicBool::new(false);

pub fn is_plugin_active() -> bool {
    PLUGIN_ACTIVE.load(Ordering::Acquire)
}

extern "C" fn init() {
    if PLUGIN_ACTIVE.load(Ordering::Acquire) {
        return;
    }

    color_backtrace::install_with_settings(
        color_backtrace::Settings::new().verbosity(color_backtrace::Verbosity::Full),
    );

    logger::initialize(true, false);

    debug!("init: modules::load()");
    modules::load();

    PLUGIN_ACTIVE.store(true, Ordering::Release);
}

extern "C" fn reset() {
    if PLUGIN_ACTIVE.swap(false, Ordering::AcqRel) {
        debug!("reset: modules::unload()");
        modules::unload();
    }

    debug!("reset: modules::load()");
    modules::load();

    PLUGIN_ACTIVE.store(true, Ordering::Release);
}

extern "C" fn free() {
    if !PLUGIN_ACTIVE.swap(false, Ordering::AcqRel) {
        return;
    }

    debug!("free: modules::unload()");
    modules::unload();
}

#[allow(non_upper_case_globals)]
#[unsafe(no_mangle)]
pub static Plugin_ApiVersion: c_int = 1;

#[allow(non_upper_case_globals)]
#[unsafe(no_mangle)]
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
