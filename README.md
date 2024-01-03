# FerroDMG

This is a GameBoy emulator, written in Rust.

## Supported Platforms

- Windows
- MacOS
- Linux

## Build Dependencies

### Linux

As this project uses [Muda](https://github.com/tauri-apps/muda), you will need to install the following dependencies:

#### Arch Linux / Manjaro:

```bash
pacman -S gtk3 xdotool
```

#### Debian / Ubuntu:

```bash
sudo apt install libgtk-3-dev libxdo-dev
```

### Other Platforms

This should build without any additional dependencies on Windows and MacOS.

## Building

To build the project, use the following command:

```bash
cargo build
```

To run the project, use the following command:

```bash
cargo run
```

## License

[Apache License 2.0](./LICENSE)
