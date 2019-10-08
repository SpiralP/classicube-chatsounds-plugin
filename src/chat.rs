use crate::option::{CHAT_KEY, SEND_CHAT_KEY};
use classicube::{
  Key_, Key__KEY_0, Key__KEY_9, Key__KEY_A, Key__KEY_BACKSPACE, Key__KEY_ESCAPE, Key__KEY_KP_ENTER,
  Key__KEY_SLASH, Key__KEY_SPACE, Key__KEY_Z,
};
use lazy_static::lazy_static;
use parking_lot::Mutex;

lazy_static! {
  pub static ref CHAT: Mutex<Chat> = Mutex::new(Chat::new());
}

pub struct Chat {
  open: bool,
  text: Vec<u8>,
}
impl Chat {
  pub fn new() -> Self {
    Self {
      text: Vec::new(),
      open: false,
    }
  }
  pub fn is_open(&self) -> bool {
    self.open
  }

  pub fn get_text(&self) -> String {
    String::from_utf8_lossy(&self.text).to_string()
  }

  pub fn handle_key_down(&mut self, key: Key_, repeat: bool) {
    if !repeat {
      if !self.open && (key == CHAT_KEY.unwrap_or(0) || key == Key__KEY_SLASH) {
        // print("OPEN");

        self.open = true;
        self.text.clear();
        return;
      }

      if key == SEND_CHAT_KEY.unwrap_or(0) || key == Key__KEY_KP_ENTER || key == Key__KEY_ESCAPE {
        // print("CLOSE");

        self.open = false;
        self.text.clear();
        return;
      }
    }
  }

  pub fn handle_key_press(&mut self, key: Key_) {
    if self.open {
      // TODO ' and other symbols!
      // TODO shift + 2 should be @?

      if (key >= Key__KEY_A && key <= Key__KEY_Z) || (key >= Key__KEY_0 && key <= Key__KEY_9) {
        let chr = key as u8;
        self.text.push(chr);
      } else if key == Key__KEY_BACKSPACE {
        self.text.pop();
      } else if key == Key__KEY_SPACE {
        self.text.push(b' ');

        // TODO delete/cursor pos :sob:
        // } else if key == Key__KEY_DELETE {
      }

      // print(self.get_text());
    }
  }
}
