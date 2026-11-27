use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io;
use std::os::fd::AsRawFd;
use std::path::{Path, PathBuf};

const EVIOCSREP: std::os::raw::c_ulong = 0x4008_4503;

#[derive(Default)]
pub struct MappingStore {
    mappings: BTreeMap<(PathBuf, u16), u16>,
}

impl MappingStore {
    pub fn load() -> io::Result<Self> {
        let path = config_path()?;
        let contents = match fs::read_to_string(path) {
            Ok(contents) => contents,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(error) => return Err(error),
        };
        let mut store = Self::default();
        for (line_number, line) in contents.lines().enumerate() {
            let fields: Vec<_> = line.split('\t').collect();
            if fields.len() != 3 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("invalid mapping on config line {}", line_number + 1),
                ));
            }
            let from = fields[1].parse().map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidData, "invalid source key code")
            })?;
            let to = fields[2].parse().map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidData, "invalid destination key code")
            })?;
            store.set(PathBuf::from(fields[0]), from, to);
        }
        Ok(store)
    }

    pub fn set(&mut self, device: PathBuf, from: u16, to: u16) {
        self.mappings.insert((device, from), to);
    }

    pub fn translate(&self, device: &Path, code: u16) -> u16 {
        self.mappings
            .get(&(device.to_path_buf(), code))
            .copied()
            .unwrap_or(code)
    }

    pub fn save(&self) -> io::Result<()> {
        let path = config_path()?;
        let parent = path.parent().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "invalid config directory")
        })?;
        fs::create_dir_all(parent)?;
        let mut contents = String::new();
        for ((device, from), to) in &self.mappings {
            contents.push_str(&format!("{}\t{from}\t{to}\n", device.display()));
        }
        let temp = path.with_extension("tmp");
        fs::write(&temp, contents)?;
        fs::rename(temp, path)
    }
}

fn config_path() -> io::Result<PathBuf> {
    let base = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "HOME is not set"))?;
    Ok(base.join("idk").join("mappings"))
}

pub fn set_repeat(device: &Path, delay_ms: u32, interval_ms: u32) -> io::Result<()> {
    let file = fs::OpenOptions::new().read(true).write(true).open(device)?;
    let repeat = [delay_ms, interval_ms];
    let result = unsafe { ioctl(file.as_raw_fd(), EVIOCSREP, repeat.as_ptr()) };
    if result < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

unsafe extern "C" {
    fn ioctl(fd: std::os::raw::c_int, request: std::os::raw::c_ulong, ...) -> std::os::raw::c_int;
}

pub fn key_code(value: &str) -> Result<u16, String> {
    if let Ok(code) = value.parse() {
        return Ok(code);
    }
    match value.to_ascii_uppercase().as_str() {
        "KEY_ESC" | "ESC" => Ok(1),
        "KEY_1" => Ok(2),
        "KEY_2" => Ok(3),
        "KEY_3" => Ok(4),
        "KEY_4" => Ok(5),
        "KEY_5" => Ok(6),
        "KEY_6" => Ok(7),
        "KEY_7" => Ok(8),
        "KEY_8" => Ok(9),
        "KEY_9" => Ok(10),
        "KEY_0" => Ok(11),
        "KEY_BACKSPACE" | "BACKSPACE" => Ok(14),
        "KEY_TAB" | "TAB" => Ok(15),
        "KEY_ENTER" | "ENTER" => Ok(28),
        "KEY_CAPSLOCK" | "CAPSLOCK" => Ok(58),
        "KEY_SPACE" | "SPACE" => Ok(57),
        "KEY_LEFTCTRL" | "LEFTCTRL" => Ok(29),
        "KEY_LEFTSHIFT" | "LEFTSHIFT" => Ok(42),
        "KEY_LEFTALT" | "LEFTALT" => Ok(56),
        "KEY_RIGHTCTRL" | "RIGHTCTRL" => Ok(97),
        "KEY_RIGHTSHIFT" | "RIGHTSHIFT" => Ok(54),
        "KEY_RIGHTALT" | "RIGHTALT" => Ok(100),
        "KEY_A" => Ok(30),
        "KEY_B" => Ok(48),
        "KEY_C" => Ok(46),
        "KEY_D" => Ok(32),
        "KEY_E" => Ok(18),
        "KEY_F" => Ok(33),
        "KEY_G" => Ok(34),
        "KEY_H" => Ok(35),
        "KEY_I" => Ok(23),
        "KEY_J" => Ok(36),
        "KEY_K" => Ok(37),
        "KEY_L" => Ok(38),
        "KEY_M" => Ok(50),
        "KEY_N" => Ok(49),
        "KEY_O" => Ok(24),
        "KEY_P" => Ok(25),
        "KEY_Q" => Ok(16),
        "KEY_R" => Ok(19),
        "KEY_S" => Ok(31),
        "KEY_T" => Ok(20),
        "KEY_U" => Ok(22),
        "KEY_V" => Ok(47),
        "KEY_W" => Ok(17),
        "KEY_X" => Ok(45),
        "KEY_Y" => Ok(21),
        "KEY_Z" => Ok(44),
        _ => Err(format!("unknown key: {value}")),
    }
}

pub fn key_name(code: u16) -> String {
    match code {
        1 => "KEY_ESC".into(),
        2 => "KEY_1".into(),
        3 => "KEY_2".into(),
        4 => "KEY_3".into(),
        5 => "KEY_4".into(),
        6 => "KEY_5".into(),
        7 => "KEY_6".into(),
        8 => "KEY_7".into(),
        9 => "KEY_8".into(),
        10 => "KEY_9".into(),
        11 => "KEY_0".into(),
        14 => "KEY_BACKSPACE".into(),
        15 => "KEY_TAB".into(),
        28 => "KEY_ENTER".into(),
        30 => "KEY_A".into(),
        31 => "KEY_S".into(),
        32 => "KEY_D".into(),
        33 => "KEY_F".into(),
        34 => "KEY_G".into(),
        35 => "KEY_H".into(),
        36 => "KEY_J".into(),
        37 => "KEY_K".into(),
        38 => "KEY_L".into(),
        16 => "KEY_Q".into(),
        17 => "KEY_W".into(),
        18 => "KEY_E".into(),
        19 => "KEY_R".into(),
        20 => "KEY_T".into(),
        21 => "KEY_Y".into(),
        22 => "KEY_U".into(),
        23 => "KEY_I".into(),
        24 => "KEY_O".into(),
        25 => "KEY_P".into(),
        44 => "KEY_Z".into(),
        45 => "KEY_X".into(),
        46 => "KEY_C".into(),
        47 => "KEY_V".into(),
        48 => "KEY_B".into(),
        49 => "KEY_N".into(),
        50 => "KEY_M".into(),
        58 => "KEY_CAPSLOCK".into(),
        57 => "KEY_SPACE".into(),
        _ => format!("KEY_{code}"),
    }
}

#[cfg(test)]
mod tests {
    use super::{key_code, key_name, MappingStore};
    use std::path::PathBuf;

    #[test]
    fn parses_key_names_and_numeric_codes() {
        assert_eq!(key_code("KEY_A").unwrap(), 30);
        assert_eq!(key_code("KEY_B").unwrap(), 48);
        assert_eq!(key_code("KEY_ENTER").unwrap(), 28);
        assert_eq!(key_code("42").unwrap(), 42);
        assert_eq!(key_name(48), "KEY_B");
    }

    #[test]
    fn mapping_is_scoped_to_its_device() {
        let first = PathBuf::from("/dev/input/event2");
        let other = PathBuf::from("/dev/input/event3");
        let mut store = MappingStore::default();
        store.set(first.clone(), 30, 48);
        assert_eq!(store.translate(&first, 30), 48);
        assert_eq!(store.translate(&other, 30), 30);
    }
}
