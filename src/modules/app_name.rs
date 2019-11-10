use crate::modules::Module;
use classicube_sys::{Server, String_AppendConst};
use std::ffi::CString;

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
    let append_app_name = CString::new(format!(" +cs{}", env!("CARGO_PKG_VERSION"))).unwrap();

    let c_str = append_app_name.as_ptr();
    self.app_name = Some(append_app_name);

    unsafe {
      String_AppendConst(&mut Server.AppName, c_str);
    }
  }

  fn unload(&mut self) {}
}
