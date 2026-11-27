mod control;
mod device;
mod watch;

use std::env;
use std::error::Error;

fn main() {
    if let Err(error) = run() {
        eprintln!("idk: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        print_usage();
        return Ok(());
    };

    match command.as_str() {
        "list" => {
            let devices = device::discover()?;
            if devices.is_empty() {
                println!("No evdev devices found. Check that /dev/input is available.");
            }
            for item in devices {
                println!(
                    "{:<24} {:<12} {}",
                    item.path.display(),
                    item.kind,
                    item.name
                );
            }
        }
        "info" => {
            let target = args.next().ok_or("usage: idk info DEVICE")?;
            let item = device::find(&target)?;
            println!("Device: {}", item.path.display());
            println!("Name:   {}", item.name);
            println!("Type:   {}", item.kind);
            for (label, value) in item.details {
                if !value.is_empty() {
                    println!("{label:<8}{value}");
                }
            }
        }
        "watch" => {
            let target = args.next();
            if args.next().is_some() {
                return Err("usage: idk watch [DEVICE]".into());
            }
            let devices = match target {
                Some(target) => vec![device::find(&target)?],
                None => device::discover()?,
            };
            if devices.is_empty() {
                return Err("no evdev devices found".into());
            }
            watch::watch(devices, control::MappingStore::load()?)?;
        }
        "map" => {
            let target = args.next().ok_or("usage: idk map DEVICE FROM_KEY TO_KEY")?;
            let from = args.next().ok_or("usage: idk map DEVICE FROM_KEY TO_KEY")?;
            let to = args.next().ok_or("usage: idk map DEVICE FROM_KEY TO_KEY")?;
            if args.next().is_some() {
                return Err("usage: idk map DEVICE FROM_KEY TO_KEY".into());
            }
            let item = device::find(&target)?;
            let from = control::key_code(&from)?;
            let to = control::key_code(&to)?;
            let mut mappings = control::MappingStore::load()?;
            mappings.set(item.path, from, to);
            mappings.save()?;
            println!(
                "Saved mapping for {}: {} -> {}",
                item.name,
                control::key_name(from),
                control::key_name(to)
            );
        }
        "repeat" => {
            let target = args
                .next()
                .ok_or("usage: idk repeat DEVICE DELAY_MS INTERVAL_MS")?;
            let delay = args
                .next()
                .ok_or("usage: idk repeat DEVICE DELAY_MS INTERVAL_MS")?;
            let interval = args
                .next()
                .ok_or("usage: idk repeat DEVICE DELAY_MS INTERVAL_MS")?;
            if args.next().is_some() {
                return Err("usage: idk repeat DEVICE DELAY_MS INTERVAL_MS".into());
            }
            let delay: u32 = delay.parse().map_err(|_| "DELAY_MS must be an integer")?;
            let interval: u32 = interval
                .parse()
                .map_err(|_| "INTERVAL_MS must be an integer")?;
            let item = device::find(&target)?;
            control::set_repeat(&item.path, delay, interval)?;
            println!(
                "Set key repeat for {}: delay={delay}ms interval={interval}ms",
                item.name
            );
        }
        "-h" | "--help" | "help" => print_usage(),
        _ => return Err(format!("unknown command: {command} (try `idk --help`)").into()),
    }
    Ok(())
}

fn print_usage() {
    println!(
        "idk - Linux evdev input tool\n\
         \n\
         Usage:\n\
           idk list\n\
           idk info DEVICE\n\
           idk watch [DEVICE]\n\
           idk map DEVICE FROM_KEY TO_KEY\n\
           idk repeat DEVICE DELAY_MS INTERVAL_MS\n\
         \n\
         DEVICE is an event name (event0) or /dev/input/event path.\n\
         Mappings affect events displayed by `watch`; they do not change kernel input."
    );
}
