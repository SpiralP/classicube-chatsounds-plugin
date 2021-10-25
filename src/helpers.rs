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
