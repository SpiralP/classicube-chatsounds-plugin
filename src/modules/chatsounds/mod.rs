mod entity_emitter;
mod event_listener;
mod random;

use self::event_listener::ChatsoundsEventListener;
use crate::{
  modules::{
    command::VOLUME_SETTING_NAME,
    shared::{FutureShared, SyncShared},
    EventHandlerModule, FuturesModule, Module, OptionModule, TabListModule,
  },
  printer::{print, status},
};
use chatsounds::Chatsounds;
use classicube_helpers::Entities;
use std::{fs, path::Path};

pub const VOLUME_NORMAL: f32 = 0.1;

enum Source {
  Api(&'static str, &'static str),
  Msgpack(&'static str, &'static str),
}

const SOURCES: &[Source] = &[
  Source::Api(
    "Metastruct/garrysmod-chatsounds",
    "sound/chatsounds/autoadd",
  ),
  Source::Api("PAC3-Server/chatsounds", "sounds/chatsounds"),
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

pub struct ChatsoundsModule {
  pub chatsounds: FutureShared<Chatsounds>,
}

impl ChatsoundsModule {
  pub fn new(
    mut option_module: SyncShared<OptionModule>,
    entities: SyncShared<Entities>,
    mut event_handler_module: SyncShared<EventHandlerModule>,
    tab_list_module: SyncShared<TabListModule>,
  ) -> Self {
    let volume = option_module
      .lock()
      .get(VOLUME_SETTING_NAME)
      .and_then(|s| s.parse().ok())
      .unwrap_or(1.0);

    let chatsounds = {
      if fs::metadata("plugins")
        .map(|meta| meta.is_dir())
        .unwrap_or(false)
      {
        let path = Path::new("plugins/chatsounds");
        fs::create_dir_all(path).unwrap();

        let mut chatsounds = Chatsounds::new(path);

        // TODO 0 volume shouldn't call play
        chatsounds.set_volume(VOLUME_NORMAL * volume);

        chatsounds
      } else {
        panic!("plugins not a dir?");
      }
    };

    let chatsounds = FutureShared::new(chatsounds);

    let chatsounds_event_listener =
      ChatsoundsEventListener::new(tab_list_module, entities, chatsounds.clone());
    event_handler_module
      .lock()
      .register_listener(chatsounds_event_listener);

    Self { chatsounds }
  }

  async fn load_sources(mut chatsounds: FutureShared<Chatsounds>) {
    let sources_len = SOURCES.len();
    for (i, source) in SOURCES.iter().enumerate() {
      let (repo, repo_path) = match source {
        Source::Api(repo, repo_path) | Source::Msgpack(repo, repo_path) => (repo, repo_path),
      };

      status(format!(
        "[{}/{}] fetching {} {}",
        i + 1,
        sources_len,
        repo,
        repo_path
      ));

      let mut chatsounds = chatsounds.lock().await;

      match source {
        Source::Api(repo, repo_path) => chatsounds.load_github_api(repo, repo_path).await,
        Source::Msgpack(repo, repo_path) => chatsounds.load_github_msgpack(repo, repo_path).await,
      }
    }
  }
}

impl Module for ChatsoundsModule {
  fn load(&mut self) {
    print(format!("Loading Chatsounds v{}", env!("CARGO_PKG_VERSION")));

    let chatsounds = self.chatsounds.clone();
    FuturesModule::spawn_future(async {
      status("chatsounds fetching sources...");

      ChatsoundsModule::load_sources(chatsounds).await;

      status("done fetching sources");
    });
  }

  fn unload(&mut self) {}
}
