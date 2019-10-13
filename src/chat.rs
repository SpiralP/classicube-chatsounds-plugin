use crate::{
  chatsounds::CHATSOUNDS,
  events::{simulate_char, simulate_key},
  option::{CHAT_KEY, SEND_CHAT_KEY},
  printer::{print, status_forever},
};
use classicube_sys::{
  Key_, Key__KEY_BACKSPACE, Key__KEY_DELETE, Key__KEY_DOWN, Key__KEY_END, Key__KEY_ENTER,
  Key__KEY_ESCAPE, Key__KEY_HOME, Key__KEY_KP_ENTER, Key__KEY_LCTRL, Key__KEY_LEFT, Key__KEY_RCTRL,
  Key__KEY_RIGHT, Key__KEY_SLASH, Key__KEY_TAB, Key__KEY_UP,
};
use std::{cell::RefCell, collections::HashMap, os::raw::c_int};

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
  history_restore: Option<Vec<u8>>,

  /// a full sentence to show in grey around what you've typed
  hint: Option<String>,

  held_keys: HashMap<Key_, bool>,
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
      history_restore: None,
      hint: None,
      held_keys: HashMap::new(),
    }
  }

  fn render_hint(&mut self) {
    let input = self.get_text();
    let input_len = input.len();

    if !input.trim().is_empty() {
      if let Some(chatsounds) = CHATSOUNDS.lock().as_mut() {
        let results = chatsounds.search(&input);

        if let Some(&(_pos, sentence)) = results
          .iter()
          .filter(|(_pos, sentence)| {
            // max chat input length
            sentence.len() <= 192
          })
          .nth(0)
        {
          // garbage

          self.hint = Some(sentence.to_string());
        } else {
          self.hint = None;
        }
      }
    }

    if let Some(hint) = &self.hint {
      if let Some(pos) = hint.find(&input) {
        if pos == 0 && hint.len() == input_len {
          // matched fully
          status_forever(input);
          return;
        }

        let hint_left = &hint[..pos];
        let hint_right = &hint[(pos + input_len)..];

        let mut colored_hint = input;
        let input_pos = if !hint_left.is_empty() {
          colored_hint = format!("&7{}&f{}", hint_left, colored_hint);
          hint_left.len() + 4 // 4 for &7 and &f
        } else {
          0
        };

        if !hint_right.is_empty() {
          colored_hint = format!("{}&7{}", colored_hint, hint_right);
        }

        if colored_hint.len() > 64 {
          // it will be cut off, so shift it

          if input_pos == 0 && input_len > 2 {
            // there was no left hint so just shift left

            colored_hint = colored_hint[(input_len - 2)..].to_string();
          }
        }

        status_forever(colored_hint);

        return;
      }
    }

    status_forever("");
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

    self.text = text.as_bytes().to_vec();
    self.cursor_pos = self.text.len();
  }

  fn handle_char_insert(&mut self, chr: u8) {
    if self.cursor_pos > self.text.len() {
      print(format!("panic! {} > {}", self.cursor_pos, self.text.len()));
      return;
    }

    self.text.insert(self.cursor_pos, chr);
    self.cursor_pos += 1;
  }

  #[allow(clippy::cognitive_complexity)]
  fn handle_key(&mut self, key: Key_) {
    if key == Key__KEY_LEFT {
      if self.is_ctrl_held() {
        let mut found_non_space = false;
        loop {
          if self.cursor_pos > 0 {
            if let Some(&chr) = self.text.get(self.cursor_pos - 1) {
              self.cursor_pos -= 1;

              if chr == b' ' && found_non_space {
                break;
              }

              if !found_non_space && chr != b' ' {
                found_non_space = true;
              }
            } else {
              break;
            }
          } else {
            break;
          }
        }
      } else if self.cursor_pos > 0 {
        self.cursor_pos -= 1;
      }
    } else if key == Key__KEY_RIGHT {
      if self.is_ctrl_held() {
        let mut found_space = false;
        loop {
          if self.text.len() > self.cursor_pos {
            if let Some(&chr) = self.text.get(self.cursor_pos) {
              if chr != b' ' && found_space {
                break;
              }

              if !found_space && chr == b' ' {
                found_space = true;
              }

              self.cursor_pos += 1;
            } else {
              break;
            }
          } else {
            break;
          }
        }
      } else if self.text.len() > self.cursor_pos {
        self.cursor_pos += 1;
      }
    } else if key == Key__KEY_BACKSPACE {
      if self.is_ctrl_held() {
        // ctrl-backspace remove word

        let mut found_non_space = false;
        loop {
          if self.cursor_pos > 0 {
            if let Some(&chr) = self.text.get(self.cursor_pos - 1) {
              if chr == b' ' && found_non_space {
                break;
              }

              if !found_non_space && chr != b' ' {
                found_non_space = true;
              }

              self.text.remove(self.cursor_pos - 1);
              self.cursor_pos -= 1;
            } else {
              break;
            }
          } else {
            break;
          }
        }
      } else if self.cursor_pos > 0 && self.text.get(self.cursor_pos - 1).is_some() {
        self.text.remove(self.cursor_pos - 1);
        self.cursor_pos -= 1;
      }

      self.render_hint();
    } else if key == Key__KEY_DELETE {
      if self.cursor_pos < self.text.len() && self.text.get(self.cursor_pos).is_some() {
        self.text.remove(self.cursor_pos);
      }

      self.render_hint();
    } else if key == Key__KEY_HOME {
      self.cursor_pos = 0;
    } else if key == Key__KEY_END {
      self.cursor_pos = self.text.len();
    } else if key == Key__KEY_UP {
      if self.is_ctrl_held() {
        // ??
        return;
      }

      if self.history_pos == 0 {
        self.history_restore = Some(self.text.to_vec());
      }

      if self.history_pos < self.history.len() {
        self.history_pos += 1;
        self.text = self.history[self.history.len() - self.history_pos].to_vec();
        self.cursor_pos = self.text.len();
      }

      self.render_hint();
    } else if key == Key__KEY_DOWN {
      if self.is_ctrl_held() {
        self.cursor_pos = self.text.len();
        return;
      }

      if self.history_pos > 1 {
        self.history_pos -= 1;
        self.text = self.history[self.history.len() - self.history_pos].to_vec();
      } else if self.history_pos == 1 {
        self.history_pos -= 1;
        if let Some(history_restore) = &self.history_restore {
          self.text = history_restore.to_vec();
        }
      } else if self.history_pos == 0 {
        if let Some(history_restore) = &self.history_restore {
          self.text = history_restore.to_vec();
        } else {
          self.text.clear();
        }
      }
      self.cursor_pos = self.text.len();

      self.render_hint();
    } else if key == Key__KEY_TAB {
      if let Some(hint) = &self.hint {
        let hint = hint.to_string();
        self.set_text(hint);
        self.render_hint();
      }
    }

    print(format!("{} {:?}", self.cursor_pos, self.get_text()));
  }

  pub fn handle_key_down(&mut self, key: Key_, repeat: bool) {
    if !repeat {
      let chat_key = CHAT_KEY.with(|chat_key| chat_key.get());

      if !self.open && (chat_key.map(|k| key == k).unwrap_or(false) || key == Key__KEY_SLASH) {
        self.open = true;
        self.text.clear();
        self.cursor_pos = 0;
        self.history_pos = 0;
        self.history_restore = None;

        if key == Key__KEY_SLASH {
          self.handle_char_insert(b'/');
        }

        // special case for non-abc key binds
        if key != Key__KEY_ENTER {
          self.dedupe_open_key = true;
        }

        return;
      }

      let send_chat_key = SEND_CHAT_KEY.with(|send_chat_key| send_chat_key.get());
      let chat_send_success =
        send_chat_key.map(|k| key == k).unwrap_or(false) || key == Key__KEY_KP_ENTER;

      if chat_send_success || key == Key__KEY_ESCAPE {
        if chat_send_success {
          self.history.push(self.text.to_vec());
        }

        self.open = false;

        return;
      }

      self.handle_held_keys(key, true);
    } // if !repeat

    if self.open {
      self.handle_key(key);
    }
  }

  fn handle_held_keys(&mut self, key: Key_, down: bool) {
    if key == Key__KEY_LCTRL || key == Key__KEY_RCTRL {
      self.held_keys.insert(key, down);
    }
  }

  fn is_ctrl_held(&self) -> bool {
    self
      .held_keys
      .get(&Key__KEY_LCTRL)
      .copied()
      .unwrap_or(false)
      || self
        .held_keys
        .get(&Key__KEY_RCTRL)
        .copied()
        .unwrap_or(false)
  }

  pub fn handle_key_up(&mut self, key: Key_) {
    self.handle_held_keys(key, false);
  }

  pub fn handle_key_press(&mut self, key: c_int) {
    if self.open {
      if self.dedupe_open_key {
        self.dedupe_open_key = false;
        return;
      }

      self.handle_char_insert(key as u8);

      self.render_hint();
    }
  }
}
