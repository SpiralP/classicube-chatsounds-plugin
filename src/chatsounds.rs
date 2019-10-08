use crate::{command::VOLUME_SETTING_NAME, option, printer::print, thread};
use chatsounds::Chatsounds;
use lazy_static::lazy_static;
use parking_lot::Mutex;
use std::{fs, path::Path};

pub const VOLUME_NORMAL: f32 = 0.1;

lazy_static! {
  pub static ref CHATSOUNDS: Mutex<Option<Chatsounds>> = Mutex::new(None);
}

pub fn load() {
  print("Loading chatsounds...");

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

  thread::spawn("chatsounds source loader", move || {
    if let Some(chatsounds) = CHATSOUNDS.lock().as_mut() {
      print("Metastruct/garrysmod-chatsounds");
      chatsounds.load_github_api(
        "Metastruct/garrysmod-chatsounds".to_string(),
        "sound/chatsounds/autoadd".to_string(),
      );
    }

    if let Some(chatsounds) = CHATSOUNDS.lock().as_mut() {
      print("PAC3-Server/chatsounds");
      chatsounds.load_github_api(
        "PAC3-Server/chatsounds".to_string(),
        "sounds/chatsounds".to_string(),
      );
    }

    for folder in &[
      "csgo", "css", "ep1", "ep2", "hl2", "l4d", "l4d2", "portal", "tf2",
    ] {
      if let Some(chatsounds) = CHATSOUNDS.lock().as_mut() {
        print(format!("PAC3-Server/chatsounds-valve-games {}", folder));
        chatsounds.load_github_msgpack(
          "PAC3-Server/chatsounds-valve-games".to_string(),
          folder.to_string(),
        );
      }
    }

    print("done fetching sources");
  });
}

pub fn unload() {
  *CHATSOUNDS.lock() = None;
}
