use crate::option::{CHAT_KEY, SEND_CHAT_KEY};
use classicube_sys::{
  Key_, Key__KEY_BACKSPACE, Key__KEY_DELETE, Key__KEY_END, Key__KEY_ENTER, Key__KEY_ESCAPE,
  Key__KEY_HOME, Key__KEY_KP_ENTER, Key__KEY_LEFT, Key__KEY_RIGHT, Key__KEY_SLASH,
};
use std::{cell::RefCell, os::raw::c_int};

thread_local! {
  pub static CHAT: RefCell<Chat> = RefCell::new(Chat::new());
}

pub struct Chat {
  open: bool,
  text: Vec<u8>,
  cursor_pos: usize,
  dedupe_open_key: bool,

  history: Vec<Vec<u8>>,
  history_pos: usize,
}
impl Chat {
  pub fn new() -> Self {
    Self {
      text: Vec::new(),
      open: false,
      cursor_pos: 0,
      dedupe_open_key: false,
      history: Vec::new(),
      history_pos: 0,
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
      let chat_key = CHAT_KEY.with(|chat_key| chat_key.get());
      let send_chat_key = SEND_CHAT_KEY.with(|send_chat_key| send_chat_key.get());

      if !self.open && (chat_key.map(|k| key == k).unwrap_or(false) || key == Key__KEY_SLASH) {
        self.open = true;
        self.text.clear();
        self.cursor_pos = 0;

        // special case for non-abc key binds
        if key != Key__KEY_ENTER {
          self.dedupe_open_key = true;
        }
        return;
      }

      let chat_send_success =
        send_chat_key.map(|k| key == k).unwrap_or(false) || key == Key__KEY_KP_ENTER;

      if chat_send_success || key == Key__KEY_ESCAPE {
        if chat_send_success {
          self.history.push(self.text.to_vec());
        }

        self.open = false;
        self.text.clear();
        self.cursor_pos = 0;
        return;
      }
    }

    if self.open {
      if key == Key__KEY_LEFT {
        if self.cursor_pos > 0 {
          self.cursor_pos -= 1;
        }
      } else if key == Key__KEY_RIGHT {
        if self.text.len() > self.cursor_pos {
          self.cursor_pos += 1;
        }
      } else if key == Key__KEY_BACKSPACE {
        if self.cursor_pos > 0 && self.text.get(self.cursor_pos - 1).is_some() {
          self.text.remove(self.cursor_pos - 1);
          self.cursor_pos -= 1;
        }
      } else if key == Key__KEY_DELETE {
        if self.cursor_pos < self.text.len() && self.text.get(self.cursor_pos).is_some() {
          self.text.remove(self.cursor_pos);
        }
      } else if key == Key__KEY_HOME {
        self.cursor_pos = 0;
      } else if key == Key__KEY_END {
        self.cursor_pos = self.text.len();

        // TODO
        // } else if key == Key__KEY_UP {
        //   if self.history_pos < self.history.len() {
        //   self.history_pos += 1;
        //   }

        //   let text = self.history[self.history.len() - self.history_pos];
        // } else if key == Key__KEY_DOWN {
        //   if self.history_pos > 0 {
        //     self.history_pos -= 1;
        //   }
      }

      // print(self.get_text());
    }
  }

  pub fn handle_key_press(&mut self, key: c_int) {
    if self.dedupe_open_key {
      self.dedupe_open_key = false;
      return;
    }

    if self.open {
      let chr = key as u8;
      self.text.insert(self.cursor_pos, chr);
      self.cursor_pos += 1;
    }
  }
}
