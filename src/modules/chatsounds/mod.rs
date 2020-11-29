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
use futures::prelude::*;
use std::{fs, path::Path};

pub const VOLUME_NORMAL: f32 = 0.1;

#[derive(Copy, Clone)]
enum SourceKind {
  Api,
  Msgpack,
}

#[derive(Copy, Clone)]
struct Source {
  repo: &'static str,
  repo_path: &'static str,
  kind: SourceKind,
}
impl Source {
  const fn api(repo: &'static str, repo_path: &'static str) -> Self {
    Self {
      repo,
      repo_path,
      kind: SourceKind::Api,
    }
  }

  const fn msgpack(repo: &'static str, repo_path: &'static str) -> Self {
    Self {
      repo,
      repo_path,
      kind: SourceKind::Msgpack,
    }
  }
}

const SOURCES: &[Source] = &[
  Source::api("NotAwesome2/chatsounds", "sounds"),
  Source::api(
    "Metastruct/garrysmod-chatsounds",
    "sound/chatsounds/autoadd",
  ),
  Source::api("PAC3-Server/chatsounds", "sounds/chatsounds"),
  Source::msgpack("PAC3-Server/chatsounds-valve-games", "csgo"),
  Source::msgpack("PAC3-Server/chatsounds-valve-games", "css"),
  Source::msgpack("PAC3-Server/chatsounds-valve-games", "ep1"),
  Source::msgpack("PAC3-Server/chatsounds-valve-games", "ep2"),
  Source::msgpack("PAC3-Server/chatsounds-valve-games", "hl1"),
  Source::msgpack("PAC3-Server/chatsounds-valve-games", "hl2"),
  Source::msgpack("PAC3-Server/chatsounds-valve-games", "l4d"),
  Source::msgpack("PAC3-Server/chatsounds-valve-games", "l4d2"),
  Source::msgpack("PAC3-Server/chatsounds-valve-games", "portal"),
  Source::msgpack("PAC3-Server/chatsounds-valve-games", "tf2"),
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

  async fn load_sources(chatsounds: &mut Chatsounds) {
    enum FetchedSource {
      Api(chatsounds::GitHubApiTrees),
      Msgpack(chatsounds::GitHubMsgpackEntries),
    }

    // TODO undo this weirdness when this is fixed
    // https://github.com/rust-lang/rust/issues/64552#issuecomment-669728225
    let stream: std::pin::Pin<Box<dyn Stream<Item = _> + Send>> = Box::pin(
      futures::stream::iter(SOURCES)
        .map(
          |Source {
             repo,
             repo_path,
             kind,
           }| {
            match kind {
              SourceKind::Api => chatsounds
                .fetch_github_api(repo, repo_path, false)
                .map_ok(FetchedSource::Api)
                .boxed(),

              SourceKind::Msgpack => chatsounds
                .fetch_github_msgpack(repo, repo_path, false)
                .map_ok(FetchedSource::Msgpack)
                .boxed(),
            }
            .map_ok(move |fetched_source| (*repo, *repo_path, fetched_source))
          },
        )
        .buffered(5),
    );

    let fetched = stream.try_collect::<Vec<_>>().await.unwrap();

    for (repo, repo_path, fetched_source) in fetched {
      match fetched_source {
        FetchedSource::Api(data) => {
          chatsounds.load_github_api(repo, repo_path, data).unwrap();
        }

        FetchedSource::Msgpack(data) => {
          chatsounds
            .load_github_msgpack(repo, repo_path, data)
            .unwrap();
        }
      }
    }

    //   let sources_len = SOURCES.len();
    //   for (i, source) in SOURCES.iter().enumerate() {
    //     let (repo, repo_path) = match source {
    //       Source::Api(repo, repo_path) | Source::Msgpack(repo, repo_path) => (repo, repo_path),
    //     };

    //     status(format!(
    //       "[{}/{}] fetching {} {}",
    //       i + 1,
    //       sources_len,
    //       repo,
    //       repo_path
    //     ));

    //     match source {
    //       Source::Api(repo, repo_path) => chatsounds
    //         .as_mut()
    //         .unwrap()
    //         .load_github_api(repo, repo_path, true)
    //         .await
    //         .unwrap(),
    //       Source::Msgpack(repo, repo_path) => chatsounds
    //         .as_mut()
    //         .unwrap()
    //         .load_github_msgpack(repo, repo_path, true)
    //         .await
    //         .unwrap(),
    //     }
    //   }
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
      let mut chatsounds = {
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

      status("chatsounds fetching sources...");

      ChatsoundsModule::load_sources(&mut chatsounds).await;

      status("done fetching sources");

      *chatsounds_option.lock().await = Some(chatsounds);
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
