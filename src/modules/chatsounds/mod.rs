mod entity_emitter;
mod event_listener;
pub mod random;
mod send_entity;

use std::{fmt::Display, fs, path::Path, pin::Pin};

use anyhow::{bail, Result};
use chatsounds::Chatsounds;
use classicube_helpers::{entities::Entities, tab_list::TabList};
use futures::prelude::*;
use tracing::error;

use self::event_listener::ChatsoundsEventListener;
use super::{FutureShared, SyncShared};
use crate::{
    modules::{
        command::VOLUME_SETTING_NAME, EventHandlerModule, FuturesModule, Module, OptionModule,
    },
    printer::print,
};

pub const VOLUME_NORMAL: f32 = 0.1;

#[derive(Debug)]
struct GitHubRepo {
    name: &'static str,
    path: &'static str,
}

impl Display for GitHubRepo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "repo {}/{}", self.name, self.path)
    }
}

enum Source {
    Api(GitHubRepo),
    MsgPack(GitHubRepo),
}
impl Source {
    const fn api(name: &'static str, path: &'static str) -> Source {
        Source::Api(GitHubRepo { name, path })
    }

    const fn msgpack(name: &'static str, path: &'static str) -> Source {
        Source::MsgPack(GitHubRepo { name, path })
    }
}

const SOURCES: &[Source] = &[
    Source::api("NotAwesome2/chatsounds", "sounds"),
    Source::api(
        "Metastruct/garrysmod-chatsounds",
        "sound/chatsounds/autoadd",
    ),
    Source::api("PAC3-Server/chatsounds", "sounds/chatsounds"),
    Source::api("MasterMenSilver/Astral-Dream-Things", "chatsounds"),
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
}

impl ChatsoundsModule {
    pub fn new(
        entities: SyncShared<Entities>,
        event_handler_module: SyncShared<EventHandlerModule>,
        tab_list: SyncShared<TabList>,
    ) -> Self {
        Self {
            chatsounds: FutureShared::default(),
            entities,
            event_handler_module,
            tab_list,
        }
    }

    async fn load_sources(chatsounds: &mut Chatsounds) -> Result<()> {
        enum SourceData {
            Api(chatsounds::GitHubApiTrees),
            MsgPack(chatsounds::GitHubMsgpackEntries),
        }

        let stream: Pin<Box<dyn Stream<Item = _> + Send>> = Box::pin(
            futures::stream::iter(SOURCES)
                .map(|source| match source {
                    Source::Api(repo) => chatsounds
                        .fetch_github_api(repo.name, repo.path)
                        .map_ok(SourceData::Api)
                        .map(move |result| (repo, result))
                        .boxed(),

                    Source::MsgPack(repo) => chatsounds
                        .fetch_github_msgpack(repo.name, repo.path)
                        .map_ok(SourceData::MsgPack)
                        .map(move |result| (repo, result))
                        .boxed(),
                })
                .buffered(5),
        );

        let fetched = stream.collect::<Vec<_>>().await;

        for (repo, result) in fetched {
            match result {
                Ok(data) => match data {
                    SourceData::Api(data) => {
                        chatsounds.load_github_api(repo.name, repo.path, data)?;
                    }
                    SourceData::MsgPack(data) => {
                        chatsounds.load_github_msgpack(repo.name, repo.path, data)?;
                    }
                },

                Err(e) => {
                    error!(?repo, ?e);
                    print(format!("{}{} {}", classicube_helpers::color::RED, repo, e));
                }
            }
        }

        Ok(())
    }
}

impl Module for ChatsoundsModule {
    fn load(&mut self) {
        print(format!("Loading Chatsounds v{}", env!("CARGO_PKG_VERSION")));

        let volume = OptionModule::get(VOLUME_SETTING_NAME)
            .and_then(|s| s.parse().ok())
            .unwrap_or(1.0);

        let chatsounds_option = self.chatsounds.clone();
        FuturesModule::spawn_future(async move {
            let mut chatsounds_option = chatsounds_option.lock().await;

            let future = async {
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
                        bail!("plugins is not a dir or doesn't exist");
                    }
                };

                ChatsoundsModule::load_sources(&mut chatsounds).await?;

                Ok(chatsounds)
            };

            match future.await {
                Ok(chatsounds) => {
                    *chatsounds_option = Some(chatsounds);
                }
                Err(e) => {
                    print(format!("{}{}", classicube_helpers::color::RED, e));
                }
            }

            drop(chatsounds_option);
        });

        let chatsounds_event_listener = ChatsoundsEventListener::new(
            self.chatsounds.clone(),
            self.entities.clone(),
            self.tab_list.clone(),
        );

        self.event_handler_module
            .borrow_mut()
            .register_listener(chatsounds_event_listener);

        self.tab_list.borrow_mut().on_added(|_event| {
            // whenever a new player joins, or someone changes map
            // we try to sync the random
            // resetting on map change could fix local map chat too?
            random::sync_reset();
        });
    }

    fn unload(&mut self) {}
}
