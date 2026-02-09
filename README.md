# pcli

A personal CLI toolkit and daemon for system monitoring and desktop environment integration.

## Overview

**pcli** is a Rust-based CLI tool designed for personal workflow automation. It consists of two main components:

- **pcli** - The command-line client
- **pdaemon** - A background daemon that provides system information via Unix sockets

## Features

- **Hardware Monitoring** - Real-time CPU, GPU, memory, disk, and network stats
- **Weather Information** - Fetch current weather data
- **Window Manager Integration** - Niri compositor support with IPC commands
- **Wallpaper Management** - Wallpaper selection and management
- **Shell Integration** - Custom launcher and panel controls via `qs` shell

## Installation

### Prerequisites

- Rust toolchain (2024 edition)
- Linux system with XDG_RUNTIME_DIR set
- `qs` shell (optional, for shell integration)

### Build & Install

```bash
# Clone the repository
git clone <repo-url>
cd pcli

# Build release binaries
just build

# Install to ~/.local/bin/
just install

# Or install and restart daemon
just install-and-restart
```

### Manual Installation

```bash
cargo build --release

# Install binaries
install -Dm755 ./target/release/pdaemon ~/.local/bin/pdaemon
install -Dm755 ./target/release/pcli ~/.local/bin/pcli
```

## Usage

### Start the Daemon

```bash
pdaemon &
```

The daemon creates a Unix socket at `$XDG_RUNTIME_DIR/pdaemon.sock`.

### CLI Commands

```bash
# Get hardware information
pcli hardware

# Get compositor data
pcli compositor

# Launch applications
pcli launch wallpaper-picker
pcli launch app-launcher
pcli launch extended-bar
pcli launch shell-settings

# Get weather
pcli weather

# Get window manager rules
pcli rules
```

## Architecture

- **Client-Server Model**: CLI communicates with daemon via Unix domain sockets
- **Modular Design**: Separate modules for hardware, weather, wallpaper, and WM integration
- **Desktop Environment Detection**: Automatically detects Niri compositor

## Available Just Commands

| Command | Description |
|---------|-------------|
| `just build` | Build in release mode |
| `just install` | Build and install to ~/.local/bin/ |
| `just restart-daemon` | Restart the pdaemon service |
| `just install-and-restart` | Full install and daemon restart |
| `just dev` | Build in debug mode |
| `just test` | Run tests |
| `just check` | Run cargo check and clippy |
| `just clean` | Clean build artifacts |

## Dependencies

- `miette` - Error handling
- `clap` - CLI argument parsing
- `serde` - Serialization
- `daemonize` - Daemon process management
- `gfxinfo` - GPU information
- `niri-ipc` - Niri compositor IPC
- `sysinfo` - System information
- `kdl` - KDL document format
- `reqwest` - HTTP client
