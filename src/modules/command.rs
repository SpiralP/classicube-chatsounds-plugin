use std::{cell::Cell, convert::AsRef, os::raw::c_int, ptr, slice, string::ToString};

use anyhow::{anyhow, Result};
use chatsounds::Chatsounds;
use classicube_sys::OwnedChatCommand;
use tracing::error;

use crate::{
    modules::{
        chatsounds::{random::get_rng, VOLUME_NORMAL},
        EventHandlerModule, FutureShared, FuturesModule, Module, OptionModule, SyncShared,
    },
    printer::print,
};

pub const MUTE_LOSE_FOCUS_SETTING_NAME: &str = "chatsounds-mute-lose-focus";
pub const VOLUME_SETTING_NAME: &str = "chatsounds-volume";

const MUTE_LOSE_FOCUS_COMMAND_HELP: &str =
    "&a/client chatsounds mute-lose-focus [true|false] &e(Default true)";
const PLAY_COMMAND_HELP: &str = "&a/client chatsounds play [text]";
const SH_COMMAND_HELP: &str = "&a/client chatsounds sh";
const VOLUME_COMMAND_HELP: &str = "&a/client chatsounds volume [volume] &e(Default 1.0)";

pub struct CommandModule {
    owned_command: OwnedChatCommand,
    event_handler_module: SyncShared<EventHandlerModule>,
    chatsounds: FutureShared<Option<Chatsounds>>,
}

impl CommandModule {
    pub fn new(
        event_handler_module: SyncShared<EventHandlerModule>,
        chatsounds: FutureShared<Option<Chatsounds>>,
    ) -> Self {
        let owned_command = OwnedChatCommand::new(
            "Chatsounds",
            c_command_callback,
            false,
            vec![PLAY_COMMAND_HELP, SH_COMMAND_HELP, VOLUME_COMMAND_HELP],
        );

        Self {
            owned_command,
            event_handler_module,
            chatsounds,
        }
    }

    async fn command_callback(&mut self, args: Vec<String>) -> Result<()> {
        let args: Vec<&str> = args.iter().map(AsRef::as_ref).collect();

        let mut chatsounds = self.chatsounds.lock().await;
        let chatsounds = chatsounds.as_mut().ok_or_else(|| anyhow!("no"))?;

        match args.as_slice() {
            ["mute-lose-focus"] => {
                let mute_lose_focus = OptionModule::get(MUTE_LOSE_FOCUS_SETTING_NAME)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(true);

                print(format!(
                    "{MUTE_LOSE_FOCUS_SETTING_NAME} (Currently {mute_lose_focus})"
                ));
            }

            ["mute-lose-focus", mute_lose_focus] => {
                let mute_lose_focus = mute_lose_focus.parse::<bool>()?;

                OptionModule::set(MUTE_LOSE_FOCUS_SETTING_NAME, format!("{mute_lose_focus}"));

                print(format!("&eSet mute-lose-focus to {mute_lose_focus}"));
            }

            ["play"] => {
                print(PLAY_COMMAND_HELP);
            }

            ["play", words @ ..] => {
                let text = words.join(" ");

                let _ignore_error = chatsounds.play(&text, get_rng("")).await;
            }

            ["sh"] => {
                chatsounds.stop_all();
            }

            ["volume"] => {
                let current_volume = chatsounds.volume() / VOLUME_NORMAL;

                print(format!(
                    "{VOLUME_COMMAND_HELP} (Currently {current_volume})"
                ));
            }

            ["volume", volume] => {
                let volume = volume.parse::<f32>()?;

                chatsounds.set_volume(VOLUME_NORMAL * volume);

                OptionModule::set(VOLUME_SETTING_NAME, format!("{volume}"));

                print(format!("&eSet volume to {volume}"));
            }

            _ => {
                let current_volume = chatsounds.volume() / VOLUME_NORMAL;
                print(MUTE_LOSE_FOCUS_COMMAND_HELP);
                print(PLAY_COMMAND_HELP);
                print(SH_COMMAND_HELP);
                print(format!(
                    "{VOLUME_COMMAND_HELP} (Currently {current_volume})"
                ));
            }
        }

        Ok(())
    }
}

// hacky fix because c_command_callback can't get instance
thread_local!(
    static COMMAND_MODULE: Cell<Option<*mut CommandModule>> = const { Cell::new(None) };
);

impl Module for CommandModule {
    fn load(&mut self) {
        self.owned_command.register();

        COMMAND_MODULE.with(|command_module| {
            command_module.set(Some(ptr::from_mut(self)));
        });
    }

    fn unload(&mut self) {
        COMMAND_MODULE.with(|command_module| {
            command_module.take();
        });
    }
}

unsafe extern "C" fn c_command_callback(args: *const classicube_sys::cc_string, args_count: c_int) {
    COMMAND_MODULE.with(move |maybe_ptr| {
        if let Some(ptr) = maybe_ptr.get() {
            let command_module = &mut *ptr;

            let args = slice::from_raw_parts(args, args_count.try_into().unwrap());
            let args: Vec<String> = args.iter().map(ToString::to_string).collect();

            FuturesModule::block_future(async {
                if let Err(e) = command_module.command_callback(args).await {
                    error!(?e);
                    print(format!("{}{}", classicube_helpers::color::RED, e));
                }
            });

            let mut event_handler_module = command_module.event_handler_module.borrow_mut();
            event_handler_module.handle_outgoing_events();
        }
    });
}
