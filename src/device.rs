use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Device {
    pub path: PathBuf,
    pub name: String,
    pub kind: &'static str,
    pub details: Vec<(&'static str, String)>,
}

pub fn discover() -> io::Result<Vec<Device>> {
    let input_dir = Path::new("/dev/input");
    let mut paths = Vec::new();
    for entry in fs::read_dir(input_dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.strip_prefix("event").is_some_and(|suffix| {
            !suffix.is_empty() && suffix.bytes().all(|byte| byte.is_ascii_digit())
        }) {
            paths.push(entry.path());
        }
    }
    paths.sort();
    paths.into_iter().map(|path| describe(path)).collect()
}

pub fn find(target: &str) -> io::Result<Device> {
    let path = if target.starts_with("/dev/input/") {
        PathBuf::from(target)
    } else if target.starts_with('/') {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "device path must be under /dev/input",
        ));
    } else {
        Path::new("/dev/input").join(target)
    };
    if !path.file_name().is_some_and(|name| {
        let name = name.to_string_lossy();
        name.strip_prefix("event")
            .is_some_and(|suffix| !suffix.is_empty() && suffix.bytes().all(|b| b.is_ascii_digit()))
    }) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "device must be an event node such as event3",
        ));
    }
    if !path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{} does not exist", path.display()),
        ));
    }
    describe(path)
}

fn describe(path: PathBuf) -> io::Result<Device> {
    let event_name = path.file_name().unwrap_or_default().to_string_lossy();
    let sysfs = Path::new("/sys/class/input")
        .join(event_name.as_ref())
        .join("device");
    let name = read(&sysfs.join("name")).unwrap_or_else(|| event_name.to_string());
    let capabilities = [
        ("EV", "capabilities/ev"),
        ("KEY", "capabilities/key"),
        ("REL", "capabilities/rel"),
        ("ABS", "capabilities/abs"),
        ("SW", "capabilities/sw"),
        ("FF", "capabilities/ff"),
        ("BUS", "id/bustype"),
        ("VENDOR", "id/vendor"),
        ("PRODUCT", "id/product"),
        ("VERSION", "id/version"),
        ("PHYS", "phys"),
        ("UNIQ", "uniq"),
    ];
    let details: Vec<_> = capabilities
        .iter()
        .map(|(label, relative)| (*label, read(&sysfs.join(relative)).unwrap_or_default()))
        .collect();
    let kind = classify(
        &name,
        detail(&details, "KEY"),
        detail(&details, "REL"),
        detail(&details, "ABS"),
    );
    Ok(Device {
        path,
        name,
        kind,
        details,
    })
}

fn detail<'a>(details: &'a [(&str, String)], name: &str) -> &'a str {
    details
        .iter()
        .find(|(label, _)| *label == name)
        .map_or("", |(_, value)| value)
}

fn read(path: &Path) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_owned())
}

fn classify(name: &str, key: &str, rel: &str, abs: &str) -> &'static str {
    let name = name.to_ascii_lowercase();
    if name.contains("touchpad") || name.contains("trackpad") {
        "touchpad"
    } else if name.contains("gamepad") || name.contains("joystick") {
        "gamepad"
    } else if abs != "0" && key != "0" && rel == "0" {
        "gamepad"
    } else if rel != "0" && key != "0" {
        "mouse"
    } else if key != "0" {
        "keyboard"
    } else if abs != "0" {
        "absolute"
    } else {
        "other"
    }
}

#[cfg(test)]
mod tests {
    use super::classify;

    #[test]
    fn classifies_common_input_devices() {
        assert_eq!(classify("USB Keyboard", "1", "0", "0"), "keyboard");
        assert_eq!(classify("USB Optical Mouse", "1", "1", "0"), "mouse");
        assert_eq!(classify("I2C Touchpad", "1", "1", "1"), "touchpad");
        assert_eq!(classify("Generic Controller", "1", "0", "1"), "gamepad");
    }
}
