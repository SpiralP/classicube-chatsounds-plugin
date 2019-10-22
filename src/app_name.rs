use classicube_sys::{Server, String_AppendConst};
use std::{cell::Cell, ffi::CString};

thread_local! {
  static APP_NAME: Cell<Option<CString>> = Cell::new(None);
}

pub fn load() {
  APP_NAME.with(|app_name| {
    let append_app_name = CString::new(format!(" +cs{}", env!("CARGO_PKG_VERSION"))).unwrap();

    let c_str = append_app_name.as_ptr();
    app_name.set(Some(append_app_name));

    unsafe {
      String_AppendConst(&mut Server.AppName, c_str);
    }
  });
}

pub fn unload() {
  APP_NAME.with(|app_name| {
    app_name.set(None);
  });
}
