use chatsounds::normalize_sentence;

use super::{HintRender, format_hint, search_player_names};

fn hint(pos: usize, sentence: &str) -> (usize, String) {
    (pos, sentence.to_string())
}

fn names(xs: &[&str]) -> Vec<String> {
    xs.iter().map(ToString::to_string).collect()
}

// --- format_hint ---

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
    assert_eq!(
        format_hint(&input, &hints, 0),
        HintRender::Full("foo bar".to_string()),
    );
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
// search returns matches `hint[pos..pos+input_len]`. If a future
// chatsounds bump changes normalization rules, this guard fails before
// the in-game Invalid log path ever fires.
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
// invalid path.
#[test]
fn inputs_with_normalize_artifacts_render_cleanly() {
    let hints = vec![hint(0, "weve got")];
    for raw in ["we've got", "we've  got", "we've-got", "we've_got"] {
        let input = normalize_sentence(raw);
        let result = format_hint(&input, &hints, 0);
        assert!(
            matches!(result, HintRender::Full(_) | HintRender::Colored(_)),
            "raw={raw:?} normalized={input:?} produced {result:?}",
        );
    }
}

// Player name: typing a lowercase prefix shows the real-case suffix as gray.
#[test]
fn player_case_insensitive_renders_real_case() {
    // simulate: search_player_names found "SpiralP" at pos 0 for input "spir"
    let hints = vec![hint(0, "SpiralP")];
    let input = normalize_sentence("spir"); // "spir"
    assert_eq!(
        format_hint(&input, &hints, 0),
        HintRender::Colored("Spir&7alP".to_string()),
    );
}

// Player name: exact-length match shows the real-case name in Full.
#[test]
fn player_full_match_shows_real_case() {
    let hints = vec![hint(0, "SpiralP")];
    let input = "SpiralP".to_string();
    assert_eq!(
        format_hint(&input, &hints, 0),
        HintRender::Full("SpiralP".to_string()),
    );
}

// --- search_player_names ---

#[test]
fn players_matched_case_insensitively() {
    let found = search_player_names(&names(&["SpiralP"]), "spir");
    assert_eq!(found, vec![(0, "SpiralP".to_string())]);
}

#[test]
fn players_sorted_pos_then_length() {
    // "zo" typed: "zoeyvidae" matches at 0, len 9; "zoe" matches at 0, len 3;
    // sorted: shorter name first on same pos.
    let found = search_player_names(&names(&["zoeyvidae", "zoe"]), "zo");
    assert_eq!(
        found,
        vec![(0, "zoe".to_string()), (0, "zoeyvidae".to_string())],
    );
}

// Worked example: player "zoeyvidae" should appear before chatsound "zoe" when
// "zoe" is typed, because players are prepended ahead of chatsounds in update_hints.
// Here we verify search_player_names returns zoeyvidae when "zoe" is the input.
#[test]
fn player_zoeyvidae_found_for_zoe() {
    let found = search_player_names(&names(&["zoeyvidae"]), "zoe");
    assert_eq!(found, vec![(0, "zoeyvidae".to_string())]);
}

#[test]
fn player_underscore_preserved() {
    let found = search_player_names(&names(&["Spirit_99"]), "spir");
    assert_eq!(found, vec![(0, "Spirit_99".to_string())]);
}

#[test]
fn player_mid_name_match() {
    // "alP" in "SpiralP" at pos 4
    let found = search_player_names(&names(&["SpiralP"]), "alp");
    assert_eq!(found, vec![(4, "SpiralP".to_string())]);
}

#[test]
fn player_no_match_returns_empty() {
    let found = search_player_names(&names(&["SpiralP"]), "zzz");
    assert!(found.is_empty());
}
