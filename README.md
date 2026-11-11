# Vibe-coded Time (Rust)

A small, experimental GUI timer and world-clock application written in Rust.

> **Status:** Hobby / experimental project. This was “vibe coded” and is not production-ready. Expect rough edges, incomplete features, and possible bugs.

## What this is

This project is a desktop GUI app built with:

- [`eframe`](https://crates.io/crates/eframe) / [`egui`](https://crates.io/crates/egui) for the GUI
- [`chrono`](https://crates.io/crates/chrono) for time handling

## Features

- Desktop GUI built with `egui`
- Timer / time-related functionality
- World-clock-style time display
- Simple single-binary Rust application

> Features may be incomplete or change without notice.

## Requirements

- Rust toolchain (`rustc`, `cargo`)
  - Install via [rustup](https://rustup.rs/)
- Native GUI development libraries required by `eframe`/`egui`

### Linux dependencies

On many Linux distributions you may need additional system packages. For Debian/Ubuntu-like systems, something like:

```bash
sudo apt update
sudo apt install -y \
  libxcb-render0-dev \
  libxcb-shape0-dev \
  libxcb-xfixes0-dev \
  libxkbcommon-dev \
  libssl-dev \
  pkg-config
