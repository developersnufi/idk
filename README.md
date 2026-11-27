# idk

A small, dependency-free evdev tool for Arch Linux and CachyOS. It discovers
input event nodes through `/dev/input` and device details through sysfs; it
does not require a desktop stack or a background service.

## Build and run

Install Rust with `sudo pacman -S rust`, then from this directory run:

```sh
cargo build --release
./target/release/idk list
./target/release/idk info event0
./target/release/idk watch
```

`watch DEVICE` monitors one event node. Reading `/dev/input/event*` usually
requires membership in the `input` group or elevated privileges. Group
membership takes effect after logging out and back in:

```sh
sudo usermod -aG input "$USER"
```

## Commands

- `idk list` lists event nodes and an approximate device class.
- `idk info DEVICE` prints the device name, capabilities and hardware IDs.
- `idk watch [DEVICE]` prints live key, relative and absolute events.
- `idk map DEVICE FROM_KEY TO_KEY` saves a per-device key mapping. For example:

  ```sh
  idk map event3 KEY_CAPSLOCK KEY_ESC
  idk watch event3
  ```

  Mappings are stored in `${XDG_CONFIG_HOME:-$HOME/.config}/idk/mappings` and
  are applied to key events printed by `watch`. This initial implementation
  does not alter the kernel input stream or remap keys for other applications.
  Key names such as `KEY_A`, `KEY_ENTER`, and `KEY_SPACE`, and numeric evdev

  created this shi with ai btw
  key codes are accepted.
- `idk repeat DEVICE DELAY_MS INTERVAL_MS` sets the kernel key-repeat delay and
  interval in milliseconds for that device. The device must support evdev key
  repeat and be writable; this usually requires `sudo`.
