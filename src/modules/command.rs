use std::{
    cell::{Cell, RefCell},
    convert::AsRef,
    os::raw::c_int,
    ptr, slice,
    string::ToString,
};

use anyhow::{Result, anyhow};
use chatsounds::Chatsounds;
use classicube_sys::OwnedChatCommand;
use tracing::error;

use crate::{
    is_plugin_active,
    modules::{
        EventHandlerModule, FutureShared, FuturesModule, Module, OptionModule, SyncShared,
        chatsounds::{ChatsoundsModule, VOLUME_NORMAL, random::get_rng},
        option::{AUTOCOMPLETE_SETTING_NAME, MUTE_LOSE_FOCUS_SETTING_NAME, VOLUME_SETTING_NAME},
    },
    printer::print,
};

const AUTOCOMPLETE_COMMAND_HELP: &str =
    "&a/client chatsounds autocomplete [true|false] &e(Default true)";
const MUTE_LOSE_FOCUS_COMMAND_HELP: &str =
    "&a/client chatsounds mute-lose-focus [true|false] &e(Default true)";
const PLAY_COMMAND_HELP: &str = "&a/client chatsounds play [text]";
const RELOAD_COMMAND_HELP: &str = "&a/client chatsounds reload";
const SH_COMMAND_HELP: &str = "&a/client chatsounds sh";
const VOLUME_COMMAND_HELP: &str = "&a/client chatsounds volume [volume] &e(Default 1.0)";

pub struct CommandModule {
    event_handler_module: SyncShared<EventHandlerModule>,
    chatsounds: FutureShared<Option<Chatsounds>>,
}

impl CommandModule {
    pub fn new(
        event_handler_module: SyncShared<EventHandlerModule>,
        chatsounds: FutureShared<Option<Chatsounds>>,
    ) -> Self {
        Self {
            event_handler_module,
            chatsounds,
        }
    }

    async fn command_callback(&mut self, args: Vec<String>) -> Result<()> {
        let args: Vec<&str> = args.iter().map(AsRef::as_ref).collect();

        if let ["reload"] = args.as_slice() {
            let mut chatsounds_option = self.chatsounds.lock().await;
            print("&eReloading chatsounds...");
            let mut new_chatsounds = ChatsoundsModule::new_chatsounds()?;
            ChatsoundsModule::load_sources(&mut new_chatsounds).await?;
            *chatsounds_option = Some(new_chatsounds);
            print("&aReloaded chatsounds");
            return Ok(());
        }

        let mut chatsounds = self.chatsounds.lock().await;
        let chatsounds = chatsounds.as_mut().ok_or_else(|| anyhow!("no"))?;

        match args.as_slice() {
            ["autocomplete"] => {
                let autocomplete = OptionModule::autocomplete();

                print(format!(
                    "{AUTOCOMPLETE_SETTING_NAME} (Currently {autocomplete})"
                ));
            }

            ["autocomplete", autocomplete] => {
                let autocomplete = autocomplete.parse::<bool>()?;

                OptionModule::set_autocomplete(autocomplete);

                print(format!("&eSet autocomplete to {autocomplete}"));
            }

            ["mute-lose-focus"] => {
                let mute_lose_focus = OptionModule::mute_lose_focus();

                print(format!(
                    "{MUTE_LOSE_FOCUS_SETTING_NAME} (Currently {mute_lose_focus})"
                ));
            }

            ["mute-lose-focus", mute_lose_focus] => {
                let mute_lose_focus = mute_lose_focus.parse::<bool>()?;

                OptionModule::set_mute_lose_focus(mute_lose_focus);

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
                print(AUTOCOMPLETE_COMMAND_HELP);
                print(MUTE_LOSE_FOCUS_COMMAND_HELP);
                print(PLAY_COMMAND_HELP);
                print(RELOAD_COMMAND_HELP);
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

// ClassiCube has no Commands_Unregister, so OwnedChatCommand must outlive
// every Free/Init cycle. We register it once per process and never drop it.
thread_local!(
    static OWNED_COMMAND: RefCell<Option<OwnedChatCommand>> = const { RefCell::new(None) };
);

impl Module for CommandModule {
    fn load(&mut self) {
        OWNED_COMMAND.with(|cell| {
            if cell.borrow().is_some() {
                return;
            }
            let mut cmd = OwnedChatCommand::new(
                "Chatsounds",
                c_command_callback,
                false,
                vec![
                    PLAY_COMMAND_HELP,
                    RELOAD_COMMAND_HELP,
                    SH_COMMAND_HELP,
                    VOLUME_COMMAND_HELP,
                ],
            );
            cmd.register();
            *cell.borrow_mut() = Some(cmd);
        });

        COMMAND_MODULE.with(|command_module| {
            command_module.set(Some(ptr::from_mut(self)));
        });
    }

    fn unload(&mut self) {
        COMMAND_MODULE.with(|command_module| {
            command_module.take();
        });
        // OWNED_COMMAND is intentionally not dropped — its Box<ChatCommand>
        // is still referenced by ClassiCube's cmds_head linked list.
    }
}

unsafe extern "C" fn c_command_callback(args: *const classicube_sys::cc_string, args_count: c_int) {
    if !is_plugin_active() {
        print("&eChatsounds: plugin not active (between hot-reload Free/Init); ignoring command");
        return;
    }

    COMMAND_MODULE.with(move |maybe_ptr| {
        if let Some(ptr) = maybe_ptr.get() {
            let command_module = unsafe { &mut *ptr };

            let args = unsafe { slice::from_raw_parts(args, args_count.try_into().unwrap()) };
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
