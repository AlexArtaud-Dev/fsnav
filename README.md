# fsnav - Fast Terminal File System Navigator

[![Crates.io](https://img.shields.io/crates/v/fsnav.svg)](https://crates.io/crates/fsnav)
[![CI](https://github.com/AlexArtaud-Dev/fsnav/actions/workflows/ci.yml/badge.svg)](https://github.com/AlexArtaud-Dev/fsnav/actions/workflows/ci.yml)

A fast and intuitive terminal-based file system navigator written in Rust. Navigate your directories with ease using keyboard shortcuts in a clean, visual interface.

## Features

- 🚀 **Fast Navigation**: Instant directory traversal with keyboard shortcuts
- 📁 **Visual Indicators**: Clear distinction between files and directories
- 🎯 **Intuitive Controls**: Arrow keys for navigation, Enter to open, Backspace to go up
- 🌍 **Cross-Platform**: Works on Windows, macOS, and Linux
- ⚡ **Lightweight**: Minimal dependencies, fast startup
- 🔒 **Safe**: Handles permission errors gracefully

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

| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate up/down in the file list |
| `→` / `Enter` | Enter selected directory |
| `←` / `Backspace` | Go to parent directory |
| `Esc` / `q` | Quit the application |

## Screenshots

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

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## License

This project is licensed under :

- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

## Acknowledgments

- Built with [crossterm](https://github.com/crossterm-rs/crossterm) for terminal manipulation

## Author

Alexandre Artaud - [@AlexArtaud-Dev](https://github.com/AlexArtaud-Dev) - Software Engineer

## Roadmap

### v0.2.0 - Advanced Permission Manager (Root Only)
- [X] **Interactive chmod interface**
  - Visual chmod builder with 3-digit selector (vertical movement for digits 0-7, horizontal for position)
  - Real-time permission explanation in plain English
  - Batch selection support with regex patterns
  - Multi-select files/directories with spacebar
  - Live preview showing: `rwxrwxrwx` format with color coding
  - Permission templates (e.g., "Make executable", "Web server files", "Secure private")
  - Undo/Redo functionality for permission changes

### v0.2.1 - Advanced Permission Manager (Root Only)
- [X] **Interactive chmod interface**
  - Officially restricting support to Unix-like systems only (Linux, macOS, BSD).
  - Add runtime check: if running on Windows, the program exits with a clear message recommending WSL.
  - Removing windows-latest from CI pipeline to avoid false build failures.
  - Updating documentation to state Windows is only supported through WSL.

### v0.2.2 - Advanced Permission Manager (Root Only)
- [ ] **Interactive chmod interface**
  - Fix chmod numbers to be centered in the selection boxes

### v0.3.0 - Ownership Manager (Root Only)
- [ ] **Interactive chown/chgrp interface**
  - User/Group selector with search functionality
  - Display current ownership and proposed changes
  - Recursive option with `-Rh` flag (follows symlinks safely)
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
- [ ] Bulk rename with pattern replacement
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
