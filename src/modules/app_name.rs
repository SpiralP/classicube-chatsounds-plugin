use std::{cell::Cell, ffi::CString};

use classicube_sys::{Server, String_AppendConst};

use crate::modules::Module;

pub struct AppNameModule {
    app_name: Option<CString>,
}

impl AppNameModule {
    pub fn new() -> Self {
        Self { app_name: None }
    }
}

impl Module for AppNameModule {
    fn load(&mut self) {
        thread_local!(
            static APPENDED: Cell<bool> = const { Cell::new(false) };
        );

        if APPENDED.get() {
            return;
        }

        let append_app_name = CString::new(format!(" cs{}", env!("CARGO_PKG_VERSION"))).unwrap();

        let c_str = append_app_name.as_ptr();
        self.app_name = Some(append_app_name);

        unsafe {
            String_AppendConst(&raw mut Server.AppName, c_str);
        }

        APPENDED.set(true);
    }

    fn unload(&mut self) {}
}
