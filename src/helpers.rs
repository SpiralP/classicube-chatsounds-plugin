use classicube_sys::{Camera, Vec3};
use ncollide3d::na::Vector3;
use tracing::warn;

pub fn remove_color_left(mut text: &str) -> &str {
    while text.len() >= 2 && text.get(0..1).is_some_and(|c| c == "&") {
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

pub fn is_global_csent_message(message: &str) -> Option<(&str, u8)> {
    let message = remove_color_left(message);

    if let Some(rest) = message.strip_prefix("csent ") {
        let mut split = rest.splitn(2, ' ');
        let entity_id = split.next()?.parse::<u8>().ok()?;
        let message = split.next()?.trim();

        if message.is_empty() {
            None
        } else {
            Some((message, entity_id))
        }
    } else {
        None
    }
}

pub fn vec3_to_vector3(v: &Vec3) -> Vector3<f32> {
    Vector3::new(v.x, v.y, v.z)
}

pub fn get_self_position_and_yaw() -> Option<(Vec3, f32)> {
    if unsafe { Camera.Active.is_null() } {
        warn!("Camera.Active is null!");
        return None;
    }
    let camera = unsafe { &*Camera.Active };
    let position = camera.GetPosition.map(|f| unsafe { f(0.0) })?;
    let orientation = camera.GetOrientation.map(|f| unsafe { f() })?;
    Some((position, orientation.x))
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

#[test]
fn test_is_global_csent_message() {
    for good in ["&fcsent 9 is good", "csent 9 is good"] {
        assert_eq!(
            is_global_csent_message(good),
            Some(("is good", 9)),
            "{good:?}"
        );
    }

    for bad in [
        "csent -1 is bad",
        "csent 256 is bad",
        "csent",
        "&fcsent",
        "",
        "&f",
        "&fcsents 1 2 3 is BAD",
        "csents 1 2 3 is BAD",
    ] {
        assert_eq!(is_global_csent_message(bad), None, "{bad:?}");
    }
}
