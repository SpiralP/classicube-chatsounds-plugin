use std::collections::HashMap;

use chatsounds::Chatsounds;
use classicube_sys::{
    InputButtons, InputButtons_KEY_BACKSPACE, InputButtons_KEY_DELETE, InputButtons_KEY_DOWN,
    InputButtons_KEY_END, InputButtons_KEY_ENTER, InputButtons_KEY_ESCAPE, InputButtons_KEY_HOME,
    InputButtons_KEY_KP_ENTER, InputButtons_KEY_LCTRL, InputButtons_KEY_LEFT,
    InputButtons_KEY_LSHIFT, InputButtons_KEY_RCTRL, InputButtons_KEY_RIGHT,
    InputButtons_KEY_RSHIFT, InputButtons_KEY_SLASH, InputButtons_KEY_TAB, InputButtons_KEY_UP,
};
use tracing::error;

use crate::{
    modules::{
        event_handler::{simulate_char, simulate_key},
        option::OptionModule,
        FutureShared, SyncShared,
    },
    printer::{print, status_forever},
};

pub struct Chat {
    open: bool,
    text: Vec<char>,
    cursor_pos: usize,
    dedupe_open_key: bool,

    history: Vec<Vec<char>>,
    history_pos: usize,
    history_restore: Option<Vec<char>>,

    search: Option<String>,
    hints: Option<Vec<(usize, String)>>,
    hint_pos: usize,

    held_keys: HashMap<InputButtons, bool>,

    open_chat_key: InputButtons,
    send_chat_key: InputButtons,

    chatsounds: FutureShared<Option<Chatsounds>>,
}

impl Chat {
    pub fn new(
        option_module: SyncShared<OptionModule>,
        chatsounds: FutureShared<Option<Chatsounds>>,
    ) -> Self {
        #[allow(clippy::unnecessary_cast)]
        let open_chat_key = option_module.borrow_mut().open_chat_key.unwrap_or(0 as _);
        #[allow(clippy::unnecessary_cast)]
        let send_chat_key = option_module.borrow_mut().send_chat_key.unwrap_or(0 as _);

        Self {
            text: Vec::new(),
            open: false,
            cursor_pos: 0,
            dedupe_open_key: false,
            history: Vec::new(),
            history_pos: 0,
            history_restore: None,
            search: None,
            hints: None,
            hint_pos: 0,
            held_keys: HashMap::new(),

            open_chat_key,
            send_chat_key,
            chatsounds,
        }
    }

    async fn update_hints(&mut self) {
        self.hints = None;
        self.hint_pos = 0;

        let input = self.get_text();
        let input = input.trim().to_string();

        if !input.is_empty() && input.len() >= 2 {
            if let Some(chatsounds) = self.chatsounds.lock().await.as_mut() {
                let results: Vec<_> = chatsounds
                    .search(&input)
                    .iter()
                    .filter_map(|(pos, sentence)| {
                        // max chat input length
                        const MAX_CHAT_INPUT: usize = 192;

                        if sentence.len() <= MAX_CHAT_INPUT {
                            Some((*pos, (*sentence).to_string()))
                        } else {
                            None
                        }
                    })
                    .collect();

                if !results.is_empty() {
                    self.search = Some(input);
                    self.hints = Some(results);
                }
            } else {
                error!("self.chatsounds is None");
            }
        }

        self.render_hints();
    }

    fn render_hints(&mut self) {
        if let Some(hints) = &self.hints {
            let input = self.search.as_ref().unwrap().clone();
            let input_len = input.len();

            if hints.get(self.hint_pos).is_none() {
                print(format!("panic! {} {}", self.hint_pos, hints.len()));
                return;
            }
            let (pos, hint) = &hints[self.hint_pos];
            let pos = *pos;

            let test_pos = hint.find(&input).unwrap_or(usize::max_value());
            if pos != test_pos {
                print(format!("panic! {} != {}", pos, test_pos));
                return;
            }

            if pos == 0 && hint.len() == input_len {
                // matched fully
                status_forever(input);
                return;
            }

            let hint_left = &hint[..pos];
            let hint_right = &hint[(pos + input_len)..];

            let mut colored_hint = input;
            let input_pos = if hint_left.is_empty() {
                0
            } else {
                colored_hint = format!("&7{}&f{}", hint_left, colored_hint);
                hint_left.len() + 4 // 4 for &7 and &f
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
        } else {
            status_forever("");
        }
    }

    pub fn get_text(&self) -> String {
        self.text.iter().collect()
    }

    pub fn set_text<T: Into<String>>(&mut self, text: T) {
        let text = text.into();

        simulate_key(InputButtons_KEY_END);
        for _ in 0..192 {
            simulate_key(InputButtons_KEY_BACKSPACE);
        }

        for chr in text.chars() {
            simulate_char(chr);
        }

        self.text = text.chars().collect();
        self.cursor_pos = self.text.len();
    }

    fn handle_char_insert(&mut self, chr: char) {
        if self.cursor_pos > self.text.len() {
            print(format!("panic! {} > {}", self.cursor_pos, self.text.len()));
            return;
        }

        self.text.insert(self.cursor_pos, chr);
        self.cursor_pos += 1;
    }

    #[allow(clippy::cognitive_complexity)]
    #[allow(clippy::too_many_lines)]
    async fn handle_key(&mut self, key: InputButtons) {
        if key == InputButtons_KEY_LEFT {
            if self.is_ctrl_held() {
                let mut found_non_space = false;
                loop {
                    if self.cursor_pos > 0 {
                        if let Some(&chr) = self.text.get(self.cursor_pos - 1) {
                            self.cursor_pos -= 1;

                            if chr == ' ' && found_non_space {
                                break;
                            }

                            if !found_non_space && chr != ' ' {
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
        } else if key == InputButtons_KEY_RIGHT {
            if self.is_ctrl_held() {
                let mut found_space = false;
                loop {
                    if self.text.len() > self.cursor_pos {
                        if let Some(&chr) = self.text.get(self.cursor_pos) {
                            if chr != ' ' && found_space {
                                break;
                            }

                            if !found_space && chr == ' ' {
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
        } else if key == InputButtons_KEY_BACKSPACE {
            if self.is_ctrl_held() {
                // ctrl-backspace remove word

                let mut found_non_space = false;
                loop {
                    if self.cursor_pos > 0 {
                        if let Some(&chr) = self.text.get(self.cursor_pos - 1) {
                            if chr == ' ' && found_non_space {
                                break;
                            }

                            if !found_non_space && chr != ' ' {
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

            self.update_hints().await;
        } else if key == InputButtons_KEY_DELETE {
            if self.cursor_pos < self.text.len() && self.text.get(self.cursor_pos).is_some() {
                self.text.remove(self.cursor_pos);
            }

            self.update_hints().await;
        } else if key == InputButtons_KEY_HOME {
            self.cursor_pos = 0;
        } else if key == InputButtons_KEY_END {
            self.cursor_pos = self.text.len();
        } else if key == InputButtons_KEY_UP {
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

            self.update_hints().await;
        } else if key == InputButtons_KEY_DOWN {
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

            self.update_hints().await;
        } else if key == InputButtons_KEY_TAB {
            if let Some(hints) = &self.hints {
                let hints_len = hints.len();

                if self.is_shift_held() {
                    // go in reverse

                    if self.hint_pos > 0 {
                        self.hint_pos -= 1;
                    } else {
                        self.hint_pos = hints_len - 1;
                    }
                } else if self.hint_pos + 1 < hints_len {
                    self.hint_pos += 1;
                } else {
                    self.hint_pos = 0;
                }

                // TODO if hint matches input then must tab, shift-tab to get the last item

                let show_pos = self.hint_pos.checked_sub(1).unwrap_or(hints_len - 1);

                let (_pos, sentence) = &hints[show_pos];
                let sentence = sentence.to_string();
                self.set_text(sentence);
            }

            self.render_hints();
        }
    }

    pub async fn handle_key_down(&mut self, key: InputButtons, repeat: bool) {
        if !repeat {
            if !self.open && (key == self.open_chat_key || key == InputButtons_KEY_SLASH) {
                self.open = true;
                self.text.clear();
                self.cursor_pos = 0;
                self.history_pos = 0;
                self.history_restore = None;
                self.hints = None;
                self.hint_pos = 0;

                if key == InputButtons_KEY_SLASH {
                    self.handle_char_insert('/');
                }

                // special case for non-abc key binds
                if key != InputButtons_KEY_ENTER {
                    self.dedupe_open_key = true;
                }

                self.render_hints();
                return;
            }

            let chat_send_success = key == self.send_chat_key || key == InputButtons_KEY_KP_ENTER;

            if chat_send_success || key == InputButtons_KEY_ESCAPE {
                if chat_send_success {
                    self.history.push(self.text.to_vec());
                }

                self.open = false;
                self.text.clear();
                self.cursor_pos = 0;
                self.history_pos = 0;
                self.history_restore = None;
                self.hints = None;
                self.hint_pos = 0;

                self.render_hints();

                return;
            }

            self.handle_held_keys(key, true);
        } // if !repeat

        if self.open {
            self.handle_key(key).await;
        }
    }

    fn handle_held_keys(&mut self, key: InputButtons, down: bool) {
        if key == InputButtons_KEY_LCTRL
            || key == InputButtons_KEY_RCTRL
            || key == InputButtons_KEY_LSHIFT
            || key == InputButtons_KEY_RSHIFT
        {
            self.held_keys.insert(key, down);
        }
    }

    fn is_ctrl_held(&self) -> bool {
        self.held_keys
            .get(&InputButtons_KEY_LCTRL)
            .copied()
            .unwrap_or(false)
            || self
                .held_keys
                .get(&InputButtons_KEY_RCTRL)
                .copied()
                .unwrap_or(false)
    }

    fn is_shift_held(&self) -> bool {
        self.held_keys
            .get(&InputButtons_KEY_LSHIFT)
            .copied()
            .unwrap_or(false)
            || self
                .held_keys
                .get(&InputButtons_KEY_RSHIFT)
                .copied()
                .unwrap_or(false)
    }

    pub async fn handle_key_up(&mut self, key: InputButtons) {
        self.handle_held_keys(key, false);
    }

    pub async fn handle_key_press(&mut self, key: char) {
        if self.open {
            if self.dedupe_open_key {
                self.dedupe_open_key = false;
                return;
            }

            self.handle_char_insert(key);

            self.update_hints().await;
        }
    }
}
