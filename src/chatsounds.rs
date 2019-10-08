use crate::{command::VOLUME_SETTING_NAME, option};
use chatsounds::Chatsounds;
use lazy_static::lazy_static;
use parking_lot::Mutex;
use std::{fs, path::Path};

pub const VOLUME_NORMAL: f32 = 0.1;

lazy_static! {
  pub static ref CHATSOUNDS: Mutex<Option<Chatsounds>> = Mutex::new(None);
}

pub fn load() {
  if fs::metadata("plugins")
    .map(|meta| meta.is_dir())
    .unwrap_or(false)
  {
    let path = Path::new("plugins/chatsounds");
    fs::create_dir_all(path).unwrap();

    let mut chatsounds = Chatsounds::new(path);

    let volume = option::get(VOLUME_SETTING_NAME)
      .and_then(|s| s.parse().ok())
      .unwrap_or(1.0);
    chatsounds.set_volume(VOLUME_NORMAL * volume);

    *CHATSOUNDS.lock() = Some(chatsounds);
  } else {
    panic!("UH OH");
  }
}
