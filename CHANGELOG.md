# Changelog

All notable changes to fsnav will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0] - 2025-01-20

### Added
- **Search functionality** (`Ctrl+F`)
    - Real-time file and directory search
    - Regex support with toggle (`Ctrl+R`)
    - Case-sensitive search toggle (`Ctrl+C`)
    - Content search within text files (`Ctrl+G`)
    - Navigate between results (`Ctrl+N`/`Ctrl+P`)
    - Highlights matching files with context

- **File preview panel** (`Ctrl+P`)
    - Split-screen view with 60/40 layout
    - Text file preview with syntax awareness
    - Binary file hex viewer
    - Image file information and ASCII art placeholder
    - Directory contents preview
    - File metadata display (size, permissions, MIME type)
    - Scrollable preview for large files

- **Bookmarks system** (`Ctrl+B`)
    - Save frequently accessed directories
    - Quick jump with keyboard shortcuts
    - Auto-generated shortcuts for bookmarks
    - Default bookmarks for common directories
    - Persistent storage in `~/.config/fsnav/bookmarks.json`
    - Access count tracking
    - Import/export functionality
    - Sort by frequency or name

- **Split-pane view** (`F2`)
    - Dual directory navigation
    - Vertical and horizontal split modes (`F6`)
    - Adjustable split ratio (`+`/`-`)
    - Independent navigation in each pane
    - Directory synchronization (`F5`)
    - Quick pane switching (`Tab`)
    - Selection support in both panes

### Changed
- **Command line interface**
    - Added `-v`/`--version` flag
    - Added `-h`/`--help` flag
    - Support for starting directory as argument
    - Improved help documentation

- **User interface**
    - Enhanced keyboard shortcut system
    - Better visual feedback for different modes
    - Improved status messages
    - More informative error handling

### Fixed
- Preview panel memory management
- Search result navigation accuracy
- Bookmark shortcut conflicts
- Split-pane rendering on terminal resize

### Technical
- Added `serde` and `serde_json` for bookmark persistence
- Modularized codebase with new feature modules
- Improved test coverage with unit tests
- Better separation of concerns

## [0.3.0] - 2025-01-19

### Added
- **Interactive chown/chgrp interface** (root only)
    - User/Group selector with search functionality
    - Display current ownership and proposed changes
    - Recursive option with `-R` flag support
    - Warning system for critical system files
    - Batch ownership changes with pattern matching
    - Preview mode showing all affected files before applying
    - Real-time filtering of users and groups
    - Visual indicators for selected items and current focus
    - Support for full names display alongside usernames
    - Safe ownership change validation

### Changed
- **Enhanced root mode capabilities**:
    - Added `o` key to open ownership manager interface
    - Improved root user detection and privilege handling
    - Extended keyboard shortcuts for advanced file management

### Fixed
- Improved error handling for ownership operations
- Better system user/group parsing with `.map_while(Result::ok)`
- Enhanced memory management for large user/group lists

## [0.2.2] - 2025-01-19

### Added
- **Shell spawning functionality**:
    - `S` or `Ctrl+D` spawns a new shell in the current directory
    - Shell inherits current working directory
    - Type 'exit' to return to original directory location
    - Supports system default shell or `$SHELL` environment variable

### Changed
- **Major code refactoring** for better maintainability:
    - Split monolithic `main.rs` into modular structure
    - New module organization:
        - `main.rs`: Entry point and terminal setup
        - `navigator.rs`: Core navigation logic
        - `file_entry.rs`: File and directory data structures
        - `permissions.rs`: Chmod interface
        - `ui.rs`: Rendering and UI components
    - Improved separation of concerns
    - Better testability

### Fixed
- **Chmod interface visual improvements**:
    - Permission preview now properly positioned below chmod selector box
    - Fixed missing bottom border on chmod selector
    - Improved number selector positioning
    - Enhanced visual spacing to prevent overlapping elements
    - Better overall interface aesthetics

### Documentation
- Updated README with shell spawning feature
- Improved code documentation and comments

## [0.2.1] - 2025-09-18

### Changed
- Officially restricted support to **Unix-like systems only** (Linux, macOS, BSD)
- Added runtime check: if running on Windows, the program exits with a clear message recommending **WSL**
- Removed `windows-latest` from CI pipeline to avoid false build failures
- Updated documentation to state Windows is only supported through **WSL**

## [0.2.0] - 2025-09-18

### Added
- **Interactive chmod interface** (root only)
    - Visual 3-digit permission selector with real-time preview
    - Live explanation of permissions in plain English
    - Color-coded permission display
    - Permission templates for common use cases
    - Security warnings for dangerous permissions
    - Binary representation display
    - Batch permission changes for multiple files

- **Selection modes**
    - Multi-select mode with Space key toggle
    - Pattern selection with regex support
    - Visual selection indicators
    - Batch operations on selected items

- **Enhanced file information**
    - Display file permissions in selection mode
    - Show owner and group information
    - Symlink detection and visual indicator
    - Improved permission string display

### Changed
- Refactored code structure with modular design
- Improved error handling for permission operations
- Enhanced UI with box-drawing characters
- Better color coding for different file types

### Fixed
- Borrow checker issue with parent directory navigation
- Proper handling of symlinks
- Better permission preservation when using chmod

## [0.1.0] - 2025-01-17

### Initial Release
- Basic terminal file system navigation
- Keyboard controls (arrows, Enter, Backspace, Esc/q)
- Directory breadcrumb display
- Visual indicators for files and folders
- Cross-platform support (Linux/macOS)
- Smooth scrolling for large directories
- Permission error handling
- Clean, minimalist interface