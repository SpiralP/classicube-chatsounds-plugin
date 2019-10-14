use crate::{
  command::VOLUME_SETTING_NAME,
  option,
  printer::{print, status},
  thread,
};
use chatsounds::Chatsounds;
use lazy_static::lazy_static;
use parking_lot::Mutex;
use std::{fs, path::Path};

pub const VOLUME_NORMAL: f32 = 0.1;

lazy_static! {
  pub static ref CHATSOUNDS: Mutex<Option<Chatsounds>> = Mutex::new(None);
}

pub fn load() {
  print(format!(
    "Loading Chatsounds v{}...",
    env!("CARGO_PKG_VERSION")
  ));

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

    // TODO 0 volume doesn't event play anything
    chatsounds.set_volume(VOLUME_NORMAL * volume);

    *CHATSOUNDS.lock() = Some(chatsounds);
  } else {
    panic!("UH OH");
  }

  thread::spawn("chatsounds source loader", move || {
    status("chatsounds fetching sources...");
    load_sources();
    status("done fetching sources");
  });
}

pub fn unload() {
  *CHATSOUNDS.lock() = None;
}

enum Source {
  Api(&'static str, &'static str),
  Msgpack(&'static str, &'static str),
}

const SOURCES: &[Source] = &[
  Source::Api(
    "Metastruct/garrysmod-chatsounds",
    "sound/chatsounds/autoadd",
  ),
  Source::Api("PAC3-Server/chatsounds", "sounds/chatsouds"),
  Source::Msgpack("PAC3-Server/chatsounds-valve-games", "csgo"),
  Source::Msgpack("PAC3-Server/chatsounds-valve-games", "css"),
  Source::Msgpack("PAC3-Server/chatsounds-valve-games", "ep1"),
  Source::Msgpack("PAC3-Server/chatsounds-valve-games", "ep2"),
  Source::Msgpack("PAC3-Server/chatsounds-valve-games", "hl2"),
  Source::Msgpack("PAC3-Server/chatsounds-valve-games", "l4d"),
  Source::Msgpack("PAC3-Server/chatsounds-valve-games", "l4d2"),
  Source::Msgpack("PAC3-Server/chatsounds-valve-games", "portal"),
  Source::Msgpack("PAC3-Server/chatsounds-valve-games", "tf2"),
];

fn load_sources() {
  let sources_len = SOURCES.len();
  for (i, source) in SOURCES.iter().enumerate() {
    if let Some(chatsounds) = CHATSOUNDS.lock().as_mut() {
      let (repo, repo_path) = match source {
        Source::Api(repo, repo_path) => (repo, repo_path),
        Source::Msgpack(repo, repo_path) => (repo, repo_path),
      };

      status(format!(
        "[{}/{}] fetching {} {}",
        i + 1,
        sources_len,
        repo,
        repo_path
      ));

      match source {
        Source::Api(repo, repo_path) => chatsounds.load_github_api(repo, repo_path),
        Source::Msgpack(repo, repo_path) => chatsounds.load_github_msgpack(repo, repo_path),
      }
    }
  }
}
