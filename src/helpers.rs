use classicube_sys::Vec3;

pub fn remove_color_left(mut text: &str) -> &str {
    while text.len() >= 2 && text.get(0..1).map(|c| c == "&").unwrap_or(false) {
        if let Some(trimmed) = text.get(2..) {
            text = trimmed;
        } else {
            break;
        }
    }

    text
}

pub fn is_continuation_message(mut message: &str) -> Option<&str> {
    if message.starts_with("> ") {
        message = message.get(2..)?;
        Some(remove_color_left(message))
    } else {
        None
    }
}

pub fn is_global_cs_message(message: &str) -> Option<&str> {
    let message = remove_color_left(message);

    message.strip_prefix("cs ")
}

pub fn is_global_cspos_message(message: &str) -> Option<(&str, Vec3)> {
    let message = remove_color_left(message);

    if let Some(rest) = message.strip_prefix("cspos ") {
        let mut split = rest.splitn(4, ' ');
        let x = split.next()?.parse::<f32>().ok()?;
        let y = split.next()?.parse::<f32>().ok()?;
        let z = split.next()?.parse::<f32>().ok()?;
        let message = split.next()?.trim();

        if message.is_empty() {
            None
        } else {
            Some((message, Vec3::new(x, y, z)))
        }
    } else {
        None
    }
}

#[test]
fn test_is_global_cs_message() {
    assert_eq!(is_global_cs_message("&fcs is good"), Some("is good"));
    assert_eq!(is_global_cs_message("cs is good"), Some("is good"));
    assert_eq!(is_global_cs_message("cs "), Some(""));
    assert_eq!(is_global_cs_message("&fcs "), Some(""));
    assert_eq!(is_global_cs_message("cs"), None);
    assert_eq!(is_global_cs_message(""), None);
    assert_eq!(is_global_cs_message("&f"), None);
    assert_eq!(is_global_cs_message("&fcs"), None);

    assert_eq!(is_global_cs_message("&fcss is BAD"), None);
}

#[test]
fn test_is_global_cspos_message() {
    for good in [
        "&fcspos 1 2 3 is good",
        "cspos 1 2 3 is good",
        "cspos 1.0 2.0 3.0 is good",
    ] {
        assert_eq!(
            is_global_cspos_message(good),
            Some(("is good", Vec3::new(1.0, 2.0, 3.0))),
            "{good:?}"
        );
    }

    for bad in [
        "cspos 1 2 is bad",
        "cspos 1 2 3 ",
        "&fcspos 1 2 3 ",
        "cspos",
        "&fcspos",
        "",
        "&f",
        "&fcsposs 1 2 3 is BAD",
        "csposs 1 2 3 is BAD",
    ] {
        assert_eq!(is_global_cspos_message(bad), None, "{bad:?}");
    }
}
