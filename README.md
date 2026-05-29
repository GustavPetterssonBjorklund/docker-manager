# Docker Manager

A small Tauri 2 + Svelte desktop app for managing the local Docker daemon.

## Features

- List all containers
- Filter by name, image, ID, or status
- Start and stop containers
- Inspect container metadata
- Tail recent logs

## Requirements

- Docker installed and available on the local machine
- Node.js and pnpm
- Rust toolchain
- Tauri desktop prerequisites for your platform

### Linux prerequisites

This app depends on GTK 3 and WebKitGTK system libraries through Tauri. On Debian/Ubuntu-based systems, install the native packages before running `pnpm tauri dev` or `pnpm tauri build`:

```sh
sudo apt update
sudo apt install -y libgtk-3-dev libcairo2-dev libpango1.0-dev libgdk-pixbuf-2.0-dev libwebkit2gtk-4.1-dev
```

If `pkg-config` still cannot find `cairo.pc` or `gdk-3.0.pc`, verify that the development packages are installed and that `PKG_CONFIG_PATH` includes the directory containing those `.pc` files.

## Development

```sh
pnpm install
pnpm tauri dev
```

## Build

```sh
pnpm tauri build
```

The app uses the local Docker CLI, so it expects the `docker` command to be on `PATH` when the desktop app runs.
