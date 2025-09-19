# fsnav - Fast Terminal File System Navigator

[![Crates.io](https://img.shields.io/crates/v/fsnav.svg)](https://crates.io/crates/fsnav)
[![CI](https://github.com/AlexArtaud-Dev/fsnav/actions/workflows/ci.yml/badge.svg)](https://github.com/AlexArtaud-Dev/fsnav/actions/workflows/ci.yml)

A fast and intuitive terminal-based file system navigator written in Rust. Navigate your directories with ease using keyboard shortcuts in a clean, visual interface.

## Features

- 🚀 **Fast Navigation**: Instant directory traversal with keyboard shortcuts
- 📁 **Visual Indicators**: Clear distinction between files and directories
- 🎯 **Intuitive Controls**: Arrow keys for navigation, Enter to open, Backspace to go up
- 🖥️ **Quick Shell Access**: Press `S` or `Ctrl+D` to open a shell in the current directory (type `exit` to return to fsnav)
- 🔒 **Permission Manager**: Interactive chmod interface for root users
- 🎨 **Pattern Selection**: Select multiple files using glob patterns or regex
- 🌍 **Unix-Native**: Optimized for Linux, macOS, and BSD systems
- ⚡ **Lightweight**: Minimal dependencies, fast startup
- 🔐 **Safe**: Handles permission errors gracefully

## Installation

### From crates.io

```bash
cargo install fsnav
```

### From source

```bash
git clone https://github.com/AlexArtaud-Dev/fsnav
cd fsnav
cargo build --release
# Binary will be in target/release/fsnav
```

## Usage

Simply run `fsnav` in your terminal:

```bash
fsnav
```

### Keyboard Shortcuts

#### Global
| Key | Action |
|-----|--------|
| `S` / `Ctrl+D` | Open a shell in the current directory (type `exit` to return) |

#### Standard Mode
| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate up/down in the file list |
| `→` / `Enter` | Enter selected directory |
| `←` / `Backspace` | Go to parent directory |
| `Esc` / `q` | Quit the application |

#### Root Mode (Additional Features)
| Key | Action |
|-----|--------|
| `s` | Enter selection mode |
| `Space` | Toggle selection (in selection mode) |
| `p` | Pattern selection mode |
| `c` | Open chmod interface |

#### Chmod Interface
| Key | Action |
|-----|--------|
| `←` / `→` | Navigate between permission digits |
| `↑` / `↓` | Increment/decrement permission value |
| `t` | Show permission templates |
| `Enter` | Apply permissions |
| `Esc` | Cancel without applying |

## Screenshots

### Standard Navigation
```
📂 /home/user/projects
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
   📁 ../
 > 📁 src/
   📁 tests/
   📄 Cargo.toml
   📄 README.md
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
↑↓:Navigate  →/Enter:Open  ←/Backspace:Up  Esc/q:Quit
```

### Interactive Chmod Interface (Root Only)
```
╔══════════════════════════════════════════════════════════════════════╗
║           INTERACTIVE CHMOD - Permission Manager                     ║
╚══════════════════════════════════════════════════════════════════════╝

📁 Selected: 3 item(s)

╭─────────────────────────────────────────────╮
│         OWNER      GROUP      OTHERS        │
├─────────────────────────────────────────────┤
│           ┌───┐     ┌───┐     ┌───┐        │
│           │ 7 │     │ 5 │     │ 5 │        │
│           └───┘     └───┘     └───┘        │
╰─────────────────────────────────────────────╯

📊 Permission Preview:
Owner:  R  W  X    Group:  R  ─  X    Other:  R  ─  X
Octal: 755 (Binary: 111 101 101)

💡 What this means:
👤 Owner can: read, write, execute/enter
👥 Group members can: read, execute/enter
🌍 Everyone else can: read, execute/enter
ℹ️ ✓ Standard - Safe for programs and directories
```

## Performance

- Handles directories with thousands of files smoothly
- Minimal memory footprint
- Instant response to keyboard input
- Efficient scrolling for large directories

## Compatibility

- **Operating Systems**: Linux, macOS, BSD  
  ⚠️ Windows is not supported directly. Please use [Windows Subsystem for Linux (WSL)](https://learn.microsoft.com/windows/wsl/) for full functionality.
- **Terminal Emulators**: All modern terminals supporting ANSI escape codes
- **Rust Version**: 1.70.0 or later

## Building from Source

Requirements:
- Rust 1.70.0 or later
- Cargo

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Run directly
cargo run
```

## Project Structure

```
fsnav/
├── src/
│   ├── main.rs           # Entry point and terminal setup
│   ├── navigator.rs      # Core navigation logic
│   ├── file_entry.rs     # File/directory data structures
│   ├── permissions.rs    # Chmod interface
│   └── ui.rs            # Rendering and UI components
├── Cargo.toml
├── README.md
├── CHANGELOG.md
└── LICENSE-MIT
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## License

This project is licensed under the MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

## Acknowledgments

- Built with [crossterm](https://github.com/crossterm-rs/crossterm) for terminal manipulation

## Author

Alexandre Artaud - [@AlexArtaud-Dev](https://github.com/AlexArtaud-Dev) - Software Engineer

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for a detailed list of changes between versions.

## Roadmap

### v0.3.0 - Ownership Manager (Root Only)
- [ ] **Interactive chown/chgrp interface**
    - User/Group selector with search functionality
    - Display current ownership and proposed changes
    - Recursive option with `-R` flag
    - Warning system for critical system files
    - Batch ownership changes with pattern matching
    - Preview mode showing all affected files before applying
    - History log of ownership changes

### v0.4.0 - Enhanced Navigation
- [ ] Search functionality (Ctrl+F) with regex support
- [ ] File preview panel (text, images as ASCII, file info)
- [ ] Bookmarks/favorites system (Ctrl+B to bookmark, Ctrl+G to go)
- [ ] Split-pane view for dual directory navigation

### v0.5.0 - File Operations
- [ ] Copy/Cut/Paste operations (Ctrl+C, Ctrl+X, Ctrl+V)
- [ ] Safe delete with trash support
- [ ] Bulk rename with pattern replacement (implement my own C program "mrename")
- [ ] Archive creation/extraction (zip, tar, gz)

### v0.6.0 - Customization
- [ ] Configuration file support (`~/.config/fsnav/config.toml`)
- [ ] Vim-like keybindings option
- [ ] Custom color themes
- [ ] Plugin system for extensions

### Future Features
- [ ] Network drive support (SMB, SSH/SFTP)
- [ ] File filtering by extension/size/date
- [ ] Advanced symlink visualization and management
- [ ] Integration with system clipboard
- [ ] File tagging system
- [ ] Quick actions menu (F-keys)

