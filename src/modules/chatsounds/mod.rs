mod entity_emitter;
mod event_listener;
mod random;
mod send_entity;

use self::event_listener::ChatsoundsEventListener;
use crate::{
  modules::{
    command::VOLUME_SETTING_NAME, EventHandlerModule, FuturesModule, Module, OptionModule,
  },
  printer::{print, status},
};
use chatsounds::Chatsounds;
use classicube_helpers::{
  entities::Entities,
  shared::{FutureShared, SyncShared},
  tab_list::TabList,
};
use std::{fs, path::Path};

pub const VOLUME_NORMAL: f32 = 0.1;

enum Source {
  Api(&'static str, &'static str),
  Msgpack(&'static str, &'static str),
}

const SOURCES: &[Source] = &[
  Source::Api("NotAwesome2/chatsounds", "sounds"),
  Source::Api(
    "Metastruct/garrysmod-chatsounds",
    "sound/chatsounds/autoadd",
  ),
  Source::Api("PAC3-Server/chatsounds", "sounds/chatsounds"),
  Source::Msgpack("PAC3-Server/chatsounds-valve-games", "csgo"),
  Source::Msgpack("PAC3-Server/chatsounds-valve-games", "css"),
  Source::Msgpack("PAC3-Server/chatsounds-valve-games", "ep1"),
  Source::Msgpack("PAC3-Server/chatsounds-valve-games", "ep2"),
  Source::Msgpack("PAC3-Server/chatsounds-valve-games", "hl1"),
  Source::Msgpack("PAC3-Server/chatsounds-valve-games", "hl2"),
  Source::Msgpack("PAC3-Server/chatsounds-valve-games", "l4d"),
  Source::Msgpack("PAC3-Server/chatsounds-valve-games", "l4d2"),
  Source::Msgpack("PAC3-Server/chatsounds-valve-games", "portal"),
  Source::Msgpack("PAC3-Server/chatsounds-valve-games", "tf2"),
];

pub struct ChatsoundsModule {
  pub chatsounds: FutureShared<Option<Chatsounds>>,
  entities: SyncShared<Entities>,
  event_handler_module: SyncShared<EventHandlerModule>,
  tab_list: SyncShared<TabList>,
  option_module: SyncShared<OptionModule>,
}

impl ChatsoundsModule {
  pub fn new(
    option_module: SyncShared<OptionModule>,
    entities: SyncShared<Entities>,
    event_handler_module: SyncShared<EventHandlerModule>,
    tab_list: SyncShared<TabList>,
  ) -> Self {
    Self {
      chatsounds: FutureShared::new(None),
      entities,
      event_handler_module,
      tab_list,
      option_module,
    }
  }

  async fn load_sources(mut chatsounds: FutureShared<Option<Chatsounds>>) {
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
        Source::Api(repo, repo_path) => chatsounds
          .as_mut()
          .unwrap()
          .load_github_api(repo, repo_path, true)
          .await
          .unwrap(),
        Source::Msgpack(repo, repo_path) => chatsounds
          .as_mut()
          .unwrap()
          .load_github_msgpack(repo, repo_path, true)
          .await
          .unwrap(),
      }
    }
  }
}

impl Module for ChatsoundsModule {
  fn load(&mut self) {
    print(format!("Loading Chatsounds v{}", env!("CARGO_PKG_VERSION")));

    let volume = self
      .option_module
      .lock()
      .get(VOLUME_SETTING_NAME)
      .and_then(|s| s.parse().ok())
      .unwrap_or(1.0);

    let mut chatsounds_option = self.chatsounds.clone();
    FuturesModule::spawn_future(async move {
      let chatsounds = {
        if fs::metadata("plugins")
          .map(|meta| meta.is_dir())
          .unwrap_or(false)
        {
          let path = Path::new("plugins/chatsounds");
          fs::create_dir_all(path).unwrap();

          let mut chatsounds = Chatsounds::new(path).unwrap();

          chatsounds.set_volume(VOLUME_NORMAL * volume);

          chatsounds
        } else {
          panic!("plugins not a dir?");
        }
      };

      *chatsounds_option.lock().await = Some(chatsounds);

      status("chatsounds fetching sources...");

      ChatsoundsModule::load_sources(chatsounds_option).await;

      status("done fetching sources");
    });

    let chatsounds_event_listener = ChatsoundsEventListener::new(
      self.tab_list.clone(),
      self.entities.clone(),
      self.chatsounds.clone(),
    );

    self
      .event_handler_module
      .lock()
      .register_listener(chatsounds_event_listener);

    self.tab_list.lock().on_added(|_event| {
      // whenever a new player joins, or someone changes map
      // we try to sync the random
      // resetting on map change could fix local map chat too?
      random::sync_reset();
    });
  }

  fn unload(&mut self) {}
}
