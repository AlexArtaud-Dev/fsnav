# fsnav - Fast Terminal File System Navigator

[![Crates.io](https://img.shields.io/crates/v/fsnav.svg)](https://crates.io/crates/fsnav)
[![CI](https://github.com/AlexArtaud-Dev/fsnav/actions/workflows/ci.yml/badge.svg)](https://github.com/AlexArtaud-Dev/fsnav/actions/workflows/ci.yml)

A fast and intuitive terminal-based file system navigator written in Rust, featuring advanced search, file preview, bookmarks, and split-pane navigation.

## 🚀 v0.4.0 - Enhanced Navigation Edition

The latest release brings powerful navigation features that transform how you explore your file system:

- **🔍 Smart Search**: Find files instantly with regex support and content search
- **👁️ Live Preview**: See file contents without opening them
- **📑 Bookmarks**: Save and jump to your favorite directories
- **🔲 Split-Pane**: Navigate two directories simultaneously

## Features

### Core Navigation
- 🚀 **Fast Navigation**: Instant directory traversal with keyboard shortcuts
- 🔍 **Visual Indicators**: Clear distinction between files and directories
- 🎯 **Intuitive Controls**: Arrow keys for navigation, Enter to open, Backspace to go up
- 🖥️ **Quick Shell Access**: Press `S` or `Ctrl+D` to open a shell in the current directory
- 📊 **Permission Manager**: Interactive chmod/chown interface for root users
- 🎨 **Pattern Selection**: Select multiple files using glob patterns or regex

### New in v0.4.0
- 🔎 **Advanced Search** (`Ctrl+F`)
    - Real-time search as you type
    - Regex pattern support
    - Search within file contents
    - Navigate between results

- 📄 **File Preview Panel** (`Ctrl+P`)
    - Split-screen preview
    - Syntax-aware text display
    - Binary hex viewer
    - Directory contents preview

- 📌 **Bookmarks System** (`Ctrl+B`)
    - Save frequently accessed directories
    - Keyboard shortcuts for quick access
    - Persistent storage
    - Usage tracking

- 📊 **Split-Pane View** (`F2`)
    - Dual directory navigation
    - Vertical/horizontal layouts
    - Independent or synchronized navigation

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

```bash
# Start in current directory
fsnav

# Start in specific directory
fsnav /path/to/directory

# Show help
fsnav --help

# Show version
fsnav --version
```

## Keyboard Shortcuts

### Standard Navigation
| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate up/down |
| `→` / `Enter` | Enter selected directory |
| `←` / `Backspace` | Go to parent directory |
| `S` / `Ctrl+D` | Open shell in current directory |
| `Esc` / `q` | Quit application |

### Search & Preview
| Key | Action |
|-----|--------|
| `Ctrl+F` | Enter search mode |
| `Ctrl+N` | Next search result |
| `Ctrl+P` | Previous search result (in search) / Toggle preview panel |
| `Ctrl+R` | Toggle regex mode (in search) |
| `Ctrl+C` | Toggle case-sensitive search |
| `Ctrl+G` | Search in file contents |

### Bookmarks
| Key | Action |
|-----|--------|
| `Ctrl+B` | Open bookmarks manager |
| `Ctrl+G` | Quick jump to bookmark |
| `a` | Add current directory (in bookmarks) |
| `d` | Delete bookmark (in bookmarks) |
| `r` | Rename bookmark (in bookmarks) |

### Split-Pane View
| Key | Action |
|-----|--------|
| `F2` | Toggle split-pane mode |
| `Tab` | Switch between panes |
| `F5` | Sync directories |
| `F6` | Toggle vertical/horizontal layout |
| `+` / `-` | Adjust split ratio |

### Root Mode Features
| Key | Action |
|-----|--------|
| `s` | Enter selection mode |
| `Space` | Toggle selection (in selection mode) |
| `p` | Pattern selection mode |
| `c` | Open chmod interface |
| `o` | Open chown interface |

## Screenshots

### Search Mode with Results
```
📂 /home/user/projects
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🔍 Search: config     [3 results]
[Regex: OFF] [Case: OFF] [Content: OFF]
───────────────────────────────────────
 > 📄 config.toml ← [1/3]
   📄 .config
   📁 configs/
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Ctrl+N/P: Next/Prev | Ctrl+R: Regex | Esc: Exit
```

### File Preview Panel
```
📂 /home/user/docs          │ Preview
━━━━━━━━━━━━━━━━━━━━━━━━━━│━━━━━━━━━━━━━━
   📁 ../                   │ Size: 2.4 KB
 > 📄 README.md             │ Perms: rw-r--r--
   📄 notes.txt             │ Type: text/markdown
   📁 images/               │ ─────────────────
━━━━━━━━━━━━━━━━━━━━━━━━━━│ # Project Title
↑↓: Navigate | Ctrl+P: Hide│ 
                           │ This is a sample
                           │ README file with
                           │ markdown content.
```

### Split-Pane Navigation
```
📂 /home/user              │ 📂 /home/user/projects
━━━━━━━━━━━━━━━━━━━━━━━━━━│━━━━━━━━━━━━━━━━━━━━━━
 > 📁 Documents/           │    📁 ../
   📁 Downloads/           │  > 📁 src/
   📁 Pictures/            │    📁 tests/
   📁 projects/            │    📄 Cargo.toml
━━━━━━━━━━━━━━━━━━━━━━━━━━│━━━━━━━━━━━━━━━━━━━━━━
[Left Pane Active]         Tab: Switch | F5: Sync
```

### Bookmarks Manager
```
📑 BOOKMARKS
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
[h] Home         /home/user       (accessed 42 times)
[d] Downloads    /home/user/Downloads (accessed 15 times)
[p] Projects     /home/user/projects  (accessed 38 times)
[r] Root         /                    (accessed 5 times)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Available shortcuts: a, b, c, e, f, g...
a: Add | d: Delete | r: Rename | Esc: Back
```

## Configuration

fsnav stores its configuration and bookmarks in `~/.config/fsnav/`:

- `bookmarks.json` - Saved bookmarks with usage statistics

## Performance

- **Instant Search**: Find files in milliseconds even in large directories
- **Lazy Loading**: Preview only loads when needed
- **Efficient Scrolling**: Smooth navigation through thousands of files
- **Memory Efficient**: Minimal memory footprint with smart caching

## Compatibility

- **Operating Systems**: Linux, macOS, BSD  
  ⚠️ Windows is not supported directly. Please use [WSL](https://learn.microsoft.com/windows/wsl/)
- **Terminal Emulators**: All modern terminals supporting ANSI escape codes
- **Rust Version**: 1.70.0 or later

## Building from Source

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
│   ├── ownership.rs      # Chown interface
│   ├── ui.rs            # Rendering and UI components
│   ├── search.rs        # Search functionality (v0.4.0)
│   ├── preview.rs       # File preview system (v0.4.0)
│   ├── bookmarks.rs     # Bookmarks manager (v0.4.0)
│   └── split_pane.rs    # Split-pane view (v0.4.0)
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
- Uses [serde](https://github.com/serde-rs/serde) for configuration serialization
- Regex support via [regex](https://github.com/rust-lang/regex)

## Author

Alexandre Artaud - [@AlexArtaud-Dev](https://github.com/AlexArtaud-Dev) - Software Engineer

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for a detailed list of changes between versions.

## Roadmap

### ✅ v0.4.0 - Enhanced Navigation (Released!)
- [x] Search functionality with regex support
- [x] File preview panel
- [x] Bookmarks/favorites system
- [x] Split-pane view for dual directory navigation

### v0.5.0 - File Operations (In Progress)
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
- [ ] Advanced symlink visualization
- [ ] Integration with system clipboard
- [ ] File tagging system
- [ ] Quick actions menu (F-keys)
- [ ] Terminal multiplexer integration
- [ ] Git integration with status indicators

## Star History

If you find fsnav useful, please consider giving it a star on [GitHub](https://github.com/AlexArtaud-Dev/fsnav)!