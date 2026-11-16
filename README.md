# RifL
### The first process for Linux on Rust

RifL is a minimal PID 1 (init) for Linux written in Rust. It uses only `std` and `nix`, which results in extremely low memory consumption (**~500 KB RSS**) and a tiny binary footprint.

This is my very first project in Rust and on GitHub. It is currently under active development.

## Features

*   **Ultra-lightweight:** Uses only `std` and `nix` (no `tokio`, no async runtime).
*   **Low memory footprint:** ~500 KB of RAM in QEMU.
*   **Unix socket control:** Listens on `/run/init.sock` for commands (shutdown/reboot).
*   **Process management:** Reads `/etc/init/autorun` to start services, collects zombies, and (soon) will support restarts.

## Build

Requires `musl` target for a static binary:
```bash
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl
