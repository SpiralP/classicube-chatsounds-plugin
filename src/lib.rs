use chatsounds::*;
use classicube::*;
use std::{ffi::CString, os::raw::c_int, ptr};
use winapi::um::winuser::{MessageBoxA, MB_ICONINFORMATION, MB_OK};

#[no_mangle]
pub static Plugin_ApiVersion: c_int = 1;

#[no_mangle]
pub static mut Plugin_Component: IGameComponent = IGameComponent {
  Init: Some(init),
  Free: None,
  Reset: None,
  OnNewMap: None,
  OnNewMapLoaded: None,
  next: ptr::null_mut(),
};

extern "C" fn init() {
  let mut chatsounds = Chatsounds::new(
    "Metastruct/garrysmod-chatsounds",
    "sound/chatsounds/autoadd",
  );

  {
    let lp_text = CString::new("Hello, world!").unwrap();
    let lp_caption = CString::new("MessageBox Example").unwrap();

    unsafe {
      MessageBoxA(
        std::ptr::null_mut(),
        lp_text.as_ptr(),
        lp_caption.as_ptr(),
        MB_OK | MB_ICONINFORMATION,
      );
    }
  }

  let mut found = chatsounds.find("bap");
  let mut loaded = found[0].download();
  loaded.play(&chatsounds.device);
}

#[test]
fn asdf() {
  init();
}
