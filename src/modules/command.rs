use crate::{
  modules::{
    chatsounds::VOLUME_NORMAL, EventHandlerModule, FutureShared, FuturesModule, Module,
    OptionModule, SyncShared,
  },
  printer::print,
};
use chatsounds::Chatsounds;
use classicube_sys::OwnedChatCommand;
use std::{cell::Cell, os::raw::c_int, slice};

// TODO move file to helpers

pub const VOLUME_SETTING_NAME: &str = "chatsounds-volume";
const VOLUME_COMMAND_HELP: &str = "&a/client chatsounds volume [volume] &e(Default 1.0)";
const SH_COMMAND_HELP: &str = "&a/client chatsounds sh";

pub struct CommandModule {
  owned_command: OwnedChatCommand,
  option_module: SyncShared<OptionModule>,
  event_handler_module: SyncShared<EventHandlerModule>,
  chatsounds: FutureShared<Option<Chatsounds>>,
}

impl CommandModule {
  pub fn new(
    option_module: SyncShared<OptionModule>,
    event_handler_module: SyncShared<EventHandlerModule>,
    chatsounds: FutureShared<Option<Chatsounds>>,
  ) -> Self {
    let owned_command = OwnedChatCommand::new(
      "Chatsounds",
      c_command_callback,
      false,
      vec![VOLUME_COMMAND_HELP, SH_COMMAND_HELP],
    );

    Self {
      owned_command,
      option_module,
      event_handler_module,
      chatsounds,
    }
  }

  async fn command_callback(&mut self, args: Vec<String>) {
    let args: Vec<&str> = args.iter().map(|s| s.as_ref()).collect();

    match args.as_slice() {
      ["volume"] => {
        let current_volume =
          self.chatsounds.lock().await.as_mut().unwrap().volume() / VOLUME_NORMAL;
        print(format!(
          "{} (Currently {})",
          VOLUME_COMMAND_HELP, current_volume
        ));
      }

      ["volume", volume] => {
        let volume_maybe: Result<f32, _> = volume.parse();
        match volume_maybe {
          Ok(volume) => {
            print(format!("&eSetting volume to {}", volume));

            self
              .chatsounds
              .lock()
              .await
              .as_mut()
              .unwrap()
              .set_volume(VOLUME_NORMAL * volume);

            self
              .option_module
              .lock()
              .set(VOLUME_SETTING_NAME, format!("{}", volume));
          }
          Err(e) => {
            print(format!("&c{}", e));
          }
        }
      }

      ["sh"] => {
        self.chatsounds.lock().await.as_mut().unwrap().stop_all();
      }

      _ => {
        let current_volume =
          self.chatsounds.lock().await.as_mut().unwrap().volume() / VOLUME_NORMAL;
        print(format!(
          "{} (Currently {})",
          VOLUME_COMMAND_HELP, current_volume
        ));
        print(SH_COMMAND_HELP);
      }
    }
  }
}

// hacky fix because c_command_callback can't get instance
thread_local!(
  static COMMAND_MODULE: Cell<Option<*mut CommandModule>> = Cell::new(None);
);

impl Module for CommandModule {
  fn load(&mut self) {
    self.owned_command.register();

    COMMAND_MODULE.with(|command_module| {
      command_module.set(Some(self as _));
    });
  }

  fn unload(&mut self) {
    COMMAND_MODULE.with(|command_module| {
      command_module.take();
    });
  }
}

unsafe extern "C" fn c_command_callback(args: *const classicube_sys::String, args_count: c_int) {
  COMMAND_MODULE.with(move |maybe_ptr| {
    if let Some(ptr) = maybe_ptr.get() {
      let command_module = &mut *ptr;

      let args = slice::from_raw_parts(args, args_count as _);
      let args: Vec<String> = args.iter().map(|cc_string| cc_string.to_string()).collect();

      FuturesModule::block_future(async {
        command_module.command_callback(args).await;
      });

      let mut event_handler_module = command_module.event_handler_module.lock();
      event_handler_module.handle_outgoing_events();
    }
  });
}
