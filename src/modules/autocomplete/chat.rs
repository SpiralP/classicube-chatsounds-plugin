use std::collections::HashMap;

use chatsounds::{Chatsounds, normalize_sentence};
use classicube_sys::{
    InputButtons, InputButtons_CCKEY_BACKSPACE, InputButtons_CCKEY_DELETE, InputButtons_CCKEY_DOWN,
    InputButtons_CCKEY_END, InputButtons_CCKEY_ENTER, InputButtons_CCKEY_ESCAPE,
    InputButtons_CCKEY_HOME, InputButtons_CCKEY_KP_ENTER, InputButtons_CCKEY_LCTRL,
    InputButtons_CCKEY_LEFT, InputButtons_CCKEY_LSHIFT, InputButtons_CCKEY_RCTRL,
    InputButtons_CCKEY_RIGHT, InputButtons_CCKEY_RSHIFT, InputButtons_CCKEY_SLASH,
    InputButtons_CCKEY_TAB, InputButtons_CCKEY_UP,
};
use tracing::error;

use crate::{
    modules::{
        FutureShared, SyncShared,
        event_handler::{simulate_char, simulate_key},
        option::OptionModule,
    },
    printer::status_forever,
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

#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
enum HintRender {
    OutOfBounds { hint_pos: usize, hints_len: usize },
    Mismatch { pos: usize, test_pos: usize },
    Full,
    Colored(String),
}

fn format_hint(input: &str, hints: &[(usize, String)], hint_pos: usize) -> HintRender {
    let input_len = input.len();

    let Some((pos, hint)) = hints.get(hint_pos) else {
        return HintRender::OutOfBounds {
            hint_pos,
            hints_len: hints.len(),
        };
    };
    let pos = *pos;

    let test_pos = hint.find(input).unwrap_or(usize::MAX);
    if pos != test_pos {
        return HintRender::Mismatch { pos, test_pos };
    }

    if pos == 0 && hint.len() == input_len {
        return HintRender::Full;
    }

    let hint_left = &hint[..pos];
    let hint_right = &hint[(pos + input_len)..];

    let mut colored_hint = input.to_string();
    let input_pos = if hint_left.is_empty() {
        0
    } else {
        colored_hint = format!("&7{hint_left}&f{colored_hint}");
        hint_left.len() + 4 // 4 for &7 and &f
    };

    if !hint_right.is_empty() {
        colored_hint = format!("{colored_hint}&7{hint_right}");
    }

    if colored_hint.len() > 64 && input_pos == 0 && input_len > 2 {
        // it will be cut off, so shift it left since there was no left hint
        colored_hint = colored_hint[(input_len - 2)..].to_string();
    }

    HintRender::Colored(colored_hint)
}

impl Chat {
    pub fn new(
        option_module: &SyncShared<OptionModule>,
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

        let input = normalize_sentence(&self.get_text());

        if input.len() >= 2 {
            if let Some(chatsounds) = self.chatsounds.lock().await.as_mut() {
                let results: Vec<_> = chatsounds
                    .search(&input)
                    .iter()
                    .filter_map(|(pos, sentence)| {
                        // max chat input length
                        const MAX_CHAT_INPUT: usize = 192;

                        if sentence.len() <= MAX_CHAT_INPUT {
                            Some((*pos, (*sentence).clone()))
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
        let Some(hints) = &self.hints else {
            status_forever("");
            return;
        };

        let input = self.search.as_ref().unwrap();
        match format_hint(input, hints, self.hint_pos) {
            HintRender::OutOfBounds {
                hint_pos,
                hints_len,
            } => error!("hint_pos {hint_pos} out of bounds (hints_len={hints_len})"),
            HintRender::Mismatch { pos, test_pos } => {
                error!("chatsounds search pos {pos} != hint.find result {test_pos}");
            }
            HintRender::Full => status_forever(input.clone()),
            HintRender::Colored(s) => status_forever(s),
        }
    }

    pub fn get_text(&self) -> String {
        self.text.iter().collect()
    }

    pub fn set_text<T: Into<String>>(&mut self, text: T) {
        let text = text.into();

        simulate_key(InputButtons_CCKEY_END);
        for _ in 0..192 {
            simulate_key(InputButtons_CCKEY_BACKSPACE);
        }

        for chr in text.chars() {
            simulate_char(chr);
        }

        self.text = text.chars().collect();
        self.cursor_pos = self.text.len();
    }

    fn handle_char_insert(&mut self, chr: char) {
        if self.cursor_pos > self.text.len() {
            error!(
                "cursor_pos {} > text.len() {}",
                self.cursor_pos,
                self.text.len()
            );
            return;
        }

        self.text.insert(self.cursor_pos, chr);
        self.cursor_pos += 1;
    }

    #[allow(clippy::cognitive_complexity)]
    #[allow(clippy::too_many_lines)]
    async fn handle_key(&mut self, key: InputButtons) {
        if key == InputButtons_CCKEY_LEFT {
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
        } else if key == InputButtons_CCKEY_RIGHT {
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
        } else if key == InputButtons_CCKEY_BACKSPACE {
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
        } else if key == InputButtons_CCKEY_DELETE {
            if self.cursor_pos < self.text.len() && self.text.get(self.cursor_pos).is_some() {
                self.text.remove(self.cursor_pos);
            }

            self.update_hints().await;
        } else if key == InputButtons_CCKEY_HOME {
            self.cursor_pos = 0;
        } else if key == InputButtons_CCKEY_END {
            self.cursor_pos = self.text.len();
        } else if key == InputButtons_CCKEY_UP {
            if self.is_ctrl_held() {
                // ??
                return;
            }

            if self.history_pos == 0 {
                self.history_restore = Some(self.text.clone());
            }

            if self.history_pos < self.history.len() {
                self.history_pos += 1;
                self.text = self.history[self.history.len() - self.history_pos].clone();
                self.cursor_pos = self.text.len();
            }

            self.update_hints().await;
        } else if key == InputButtons_CCKEY_DOWN {
            if self.is_ctrl_held() {
                self.cursor_pos = self.text.len();
                return;
            }

            if self.history_pos > 1 {
                self.history_pos -= 1;
                self.text = self.history[self.history.len() - self.history_pos].clone();
            } else if self.history_pos == 1 {
                self.history_pos -= 1;
                if let Some(history_restore) = &self.history_restore {
                    self.text = history_restore.clone();
                }
            } else if self.history_pos == 0 {
                if let Some(history_restore) = &self.history_restore {
                    self.text = history_restore.clone();
                } else {
                    self.text.clear();
                }
            }
            self.cursor_pos = self.text.len();

            self.update_hints().await;
        } else if key == InputButtons_CCKEY_TAB {
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
                let sentence = sentence.clone();
                self.set_text(sentence);
            }

            self.render_hints();
        }
    }

    pub async fn handle_key_down(&mut self, key: InputButtons, repeating: bool) {
        if !repeating {
            if !self.open && (key == self.open_chat_key || key == InputButtons_CCKEY_SLASH) {
                self.open = true;
                self.text.clear();
                self.cursor_pos = 0;
                self.history_pos = 0;
                self.history_restore = None;
                self.hints = None;
                self.hint_pos = 0;

                if key == InputButtons_CCKEY_SLASH {
                    self.handle_char_insert('/');
                }

                // special case for non-abc key binds
                if key != InputButtons_CCKEY_ENTER {
                    self.dedupe_open_key = true;
                }

                self.render_hints();
                return;
            }

            let chat_send_success = key == self.send_chat_key || key == InputButtons_CCKEY_KP_ENTER;

            if chat_send_success || key == InputButtons_CCKEY_ESCAPE {
                if chat_send_success {
                    self.history.push(self.text.clone());
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
        } // if !repeating

        if self.open {
            self.handle_key(key).await;
        }
    }

    fn handle_held_keys(&mut self, key: InputButtons, down: bool) {
        if key == InputButtons_CCKEY_LCTRL
            || key == InputButtons_CCKEY_RCTRL
            || key == InputButtons_CCKEY_LSHIFT
            || key == InputButtons_CCKEY_RSHIFT
        {
            self.held_keys.insert(key, down);
        }
    }

    fn is_ctrl_held(&self) -> bool {
        self.held_keys
            .get(&InputButtons_CCKEY_LCTRL)
            .copied()
            .unwrap_or(false)
            || self
                .held_keys
                .get(&InputButtons_CCKEY_RCTRL)
                .copied()
                .unwrap_or(false)
    }

    fn is_shift_held(&self) -> bool {
        self.held_keys
            .get(&InputButtons_CCKEY_LSHIFT)
            .copied()
            .unwrap_or(false)
            || self
                .held_keys
                .get(&InputButtons_CCKEY_RSHIFT)
                .copied()
                .unwrap_or(false)
    }

    pub fn handle_key_up(&mut self, key: InputButtons, _repeating: bool) {
        self.handle_held_keys(key, false);
    }

    pub async fn handle_key_press(&mut self, key: char) {
        if self.open {
            if self.dedupe_open_key {
                self.dedupe_open_key = false;
                return;
            }

            if key.is_alphanumeric() || key == ' ' {
                self.handle_char_insert(key);
                self.update_hints().await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use chatsounds::normalize_sentence;

    use super::{HintRender, format_hint};

    fn hint(pos: usize, sentence: &str) -> (usize, String) {
        (pos, sentence.to_string())
    }

    // Regression: chatsounds 6.0 normalizes inputs before matching, so
    // search positions refer to the normalized stored key. If the plugin
    // stores the raw typed input instead, `hint.find(raw_input)` returns
    // None whenever the raw input contains characters that normalize away
    // (`/`, `_`, `-`, `.`, `'`, etc.), surfacing as
    // `panic! <pos> != 18446744073709551615` in chat.
    #[test]
    fn raw_input_with_slash_reproduces_pre_fix_mismatch() {
        let hints = vec![hint(0, "foo bar")];
        assert_eq!(
            format_hint("/foo", &hints, 0),
            HintRender::Mismatch {
                pos: 0,
                test_pos: usize::MAX,
            },
        );
    }

    #[test]
    fn normalized_input_with_slash_renders() {
        let hints = vec![hint(0, "foo bar")];
        let input = normalize_sentence("/foo");
        assert_eq!(input, "foo");
        assert_eq!(
            format_hint(&input, &hints, 0),
            HintRender::Colored("foo&7 bar".to_string()),
        );
    }

    #[test]
    fn full_match_short_circuits() {
        let hints = vec![hint(0, "foo bar")];
        let input = normalize_sentence("foo bar");
        assert_eq!(format_hint(&input, &hints, 0), HintRender::Full);
    }

    #[test]
    fn mid_sentence_match_includes_left_context() {
        let hints = vec![hint(4, "foo bar")];
        let input = normalize_sentence("bar");
        assert_eq!(
            format_hint(&input, &hints, 0),
            HintRender::Colored("&7foo &fbar".to_string()),
        );
    }

    #[test]
    fn out_of_bounds_index_reports_oob() {
        let hints = vec![hint(0, "foo bar")];
        assert_eq!(
            format_hint("foo", &hints, 5),
            HintRender::OutOfBounds {
                hint_pos: 5,
                hints_len: 1,
            },
        );
    }

    // The fix relies on `chatsounds.search` running `normalize_sentence`
    // internally and on `normalize_sentence` being idempotent — we
    // normalize once before calling search, then trust that the position
    // search returns matches `hint.find(stored_normalized_input)`. If a
    // future chatsounds bump changes normalization rules, this guard
    // fails before the in-game `Mismatch` log path ever fires.
    #[test]
    fn normalize_sentence_is_idempotent() {
        for raw in [
            "/foo",
            "we've",
            "foo-bar",
            "FOO BAR",
            "foo  bar",
            "  leading",
            "trailing  ",
            "1,000",
            "hello_world.wav",
        ] {
            let once = normalize_sentence(raw);
            let twice = normalize_sentence(&once);
            assert_eq!(once, twice, "raw={raw:?}");
        }
    }

    // Each raw input here contains a character that `normalize_sentence`
    // rewrites — apostrophe dropped, dash/underscore/whitespace runs
    // collapsed to a single space. All should render without tripping the
    // mismatch path.
    #[test]
    fn inputs_with_normalize_artifacts_render_cleanly() {
        let hints = vec![hint(0, "weve got")];
        for raw in ["we've got", "we've  got", "we've-got", "we've_got"] {
            let input = normalize_sentence(raw);
            let result = format_hint(&input, &hints, 0);
            assert!(
                matches!(result, HintRender::Full | HintRender::Colored(_)),
                "raw={raw:?} normalized={input:?} produced {result:?}",
            );
        }
    }
}
