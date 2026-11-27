use crate::control::{key_name, MappingStore};
use crate::device::Device;
use std::fs::File;
use std::io::{self, Read};
use std::mem::size_of;
use std::os::raw::c_long;
use std::sync::{Arc, Mutex};
use std::thread;

#[repr(C)]
#[derive(Clone, Copy)]
struct InputEvent {
    seconds: c_long,
    microseconds: c_long,
    event_type: u16,
    code: u16,
    value: i32,
}

pub fn watch(devices: Vec<Device>, mappings: MappingStore) -> io::Result<()> {
    let output = Arc::new(Mutex::new(()));
    let mappings = Arc::new(mappings);
    let mut workers = Vec::new();
    for device in devices {
        let output = Arc::clone(&output);
        let mappings = Arc::clone(&mappings);
        workers.push(thread::spawn(move || {
            if let Err(error) = watch_device(&device, &mappings, &output) {
                eprintln!("{}: {error}", device.path.display());
            }
        }));
    }
    for worker in workers {
        worker
            .join()
            .map_err(|_| io::Error::other("event reader thread failed"))?;
    }
    Ok(())
}

fn watch_device(device: &Device, mappings: &MappingStore, output: &Mutex<()>) -> io::Result<()> {
    let mut file = File::open(&device.path)?;
    let mut bytes = vec![0_u8; size_of::<InputEvent>()];
    loop {
        file.read_exact(&mut bytes)?;
        let event = unsafe { std::ptr::read_unaligned(bytes.as_ptr().cast::<InputEvent>()) };
        let code = if event.event_type == 1 {
            mappings.translate(&device.path, event.code)
        } else {
            event.code
        };
        let event_name = match event.event_type {
            0 => "SYN",
            1 => "KEY",
            2 => "REL",
            3 => "ABS",
            4 => "MSC",
            5 => "SW",
            17 => "LED",
            18 => "SND",
            20 => "REP",
            21 => "FF",
            _ => "EVENT",
        };
        let _guard = output
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if event.event_type == 1 {
            println!(
                "[{}.{:06}] {} {} {} {}",
                event.seconds,
                event.microseconds,
                device.name,
                event_name,
                key_name(code),
                match event.value {
                    0 => "released",
                    1 => "pressed",
                    2 => "repeat",
                    _ => "unknown",
                }
            );
        } else {
            println!(
                "[{}.{:06}] {} {} code={} value={}",
                event.seconds, event.microseconds, device.name, event_name, event.code, event.value
            );
        }
    }
}
