use crate::{
  chatsounds::CHATSOUNDS,
  events::{simulate_char, simulate_key},
  option::{CHAT_KEY, SEND_CHAT_KEY},
  printer::{print, status, status_forever},
};
use classicube_sys::{
  Key_, Key__KEY_BACKSPACE, Key__KEY_DELETE, Key__KEY_DOWN, Key__KEY_END, Key__KEY_ENTER,
  Key__KEY_ESCAPE, Key__KEY_HOME, Key__KEY_KP_ENTER, Key__KEY_LEFT, Key__KEY_RIGHT, Key__KEY_SLASH,
  Key__KEY_TAB, Key__KEY_UP,
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
  // TODO history_pos: usize,
  /// a full sentence to show in grey around what you've typed
  hint: Option<String>,
}

impl Chat {
  pub fn new() -> Self {
    Self {
      text: Vec::new(),
      open: false,
      cursor_pos: 0,
      dedupe_open_key: false,
      history: Vec::new(),
      // history_pos: 0,
      hint: None,
    }
  }

  fn render(&mut self) {
    let input = self.get_text();

    // make sure everything in the input matches what we have
    self.set_text(&input);

    let previous_cursor_pos = self.cursor_pos;

    if let Some(hint) = &self.hint {
      if let Some(pos) = hint.find(&input) {
        if pos == 0 && hint.len() == input.len() {
          return;
        }

        let hint_left = &hint[..pos];
        let hint_right = &hint[(pos + input.len())..];

        if !hint_right.is_empty() {
          simulate_key(Key__KEY_END);
          let colored_hint = format!("&7{}", hint_right);

          for &chr in colored_hint.as_bytes() {
            simulate_char(chr);
          }
        }

        let new_pos = if !hint_left.is_empty() {
          simulate_key(Key__KEY_HOME);

          let colored_hint_left = format!("&7{}&f", hint_left);
          for &chr in colored_hint_left.as_bytes() {
            simulate_char(chr);
          }

          colored_hint_left.len()
        } else {
          0
        };

        self.set_cursor_pos(previous_cursor_pos + new_pos);
      }
    }
  }

  pub fn get_text(&self) -> String {
    String::from_utf8_lossy(&self.text).to_string()
  }

  pub fn set_text<T: Into<String>>(&mut self, text: T) {
    let text = text.into();

    simulate_key(Key__KEY_END);
    for _ in 0..192 {
      simulate_key(Key__KEY_BACKSPACE);
    }

    for &chr in text.as_bytes() {
      simulate_char(chr);
    }
  }

  pub fn set_cursor_pos(&mut self, cursor_pos: usize) {
    simulate_key(Key__KEY_HOME);

    for _ in 0..cursor_pos {
      simulate_key(Key__KEY_RIGHT);
    }
  }

  fn handle_char_insert(&mut self, chr: u8) {
    if self.cursor_pos > self.text.len() {
      print(format!("panic! {} > {}", self.cursor_pos, self.text.len()));
      return;
    }

    self.text.insert(self.cursor_pos, chr);
    self.cursor_pos += 1;

    self.render();
  }

  fn do_char_insert(&mut self, chr: u8) {
    simulate_char(chr);
    self.handle_char_insert(chr);
  }

  fn do_key(&mut self, key: Key_) {
    simulate_key(key);
    self.handle_key_down(key, false);
  }

  pub fn handle_key_down(&mut self, key: Key_, repeat: bool) -> bool {
    if !repeat {
      let chat_key = CHAT_KEY.with(|chat_key| chat_key.get());

      if !self.open && (chat_key.map(|k| key == k).unwrap_or(false) || key == Key__KEY_SLASH) {
        self.open = true;
        self.text.clear();
        self.cursor_pos = 0;

        // special case for non-abc key binds
        if key != Key__KEY_ENTER {
          self.dedupe_open_key = true;
        }

        return true;
      }

      let send_chat_key = SEND_CHAT_KEY.with(|send_chat_key| send_chat_key.get());
      let chat_send_success =
        send_chat_key.map(|k| key == k).unwrap_or(false) || key == Key__KEY_KP_ENTER;

      if chat_send_success || key == Key__KEY_ESCAPE {
        if chat_send_success {
          self.history.push(self.text.to_vec());

          let input = self.get_text();
          // make sure everything in the input matches what we have
          self.set_text(&input);
        }

        self.open = false;

        return true;
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
        // TODO ctrl-backspace word delete
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
        self.render();

        return false; // test
      } else if key == Key__KEY_END {
        self.cursor_pos = self.text.len();
        self.render();
        return false; // test
      } else if key == Key__KEY_UP || key == Key__KEY_DOWN {
        self.open = false;

      // } else if key == Key__KEY_UP {

      // TODO
      // if self.history_pos < self.history.len() {
      // self.history_pos += 1;
      // }

      // let text = self.history[self.history.len() - self.history_pos];
      // } else if key == Key__KEY_DOWN {

      // if self.history_pos > 0 {
      //   self.history_pos -= 1;
      // }
      } else if key == Key__KEY_TAB {
        if let Some(hint) = &self.hint {
          self.text = hint.as_bytes().to_vec();
        }

        self.render();

        return false; // don't handle tab because we are
      }
    }

    true
  }

  pub fn handle_key_press(&mut self, key: c_int) {
    if self.open {
      if self.dedupe_open_key {
        self.dedupe_open_key = false;
        return;
      }

      self.handle_char_insert(key as u8);

      let input = &self.get_text();

      if !input.trim().is_empty() {
        if let Some(chatsounds) = CHATSOUNDS.lock().as_mut() {
          let results = chatsounds.search(input);

          if let Some(&(_pos, sentence)) = results
            .iter()
            .filter(|(_pos, sentence)| {
              // max chat input length
              sentence.len() <= 192
            })
            .nth(0)
          {
            status_forever(sentence);

            self.hint = Some(sentence.to_string());
            self.render();
          }
        }
      }
    }
  }
}
