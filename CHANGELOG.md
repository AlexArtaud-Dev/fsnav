# Changelog

All notable changes to fsnav will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
- **Interactive chmod interface** (root only)
  - Fix of the number to be centered in the box

## [0.2.1] - 2025-09-18

### Changed
- Officially restricted support to **Unix-like systems only** (Linux, macOS, BSD).
- Added runtime check: if running on Windows, the program exits with a clear message recommending **WSL**.
- Removed `windows-latest` from CI pipeline to avoid false build failures.
- Updated documentation to state Windows is only supported through **WSL**.

## [0.2.0] - 2025-09-18

### Added
- **Interactive chmod interface** (root only)
  - Visual 3-digit permission selector with real-time preview
  - Live explanation of permissions in plain English
  - Color-coded permission display (Owner: red, Group: yellow, Others: green)
  - Permission templates for common use cases (755, 644, 600, etc.)
  - Security warnings for dangerous permissions (777, 666)
  - Binary representation display
  - Batch permission changes for multiple files

- **Selection modes**
  - Multi-select mode with Space key toggle
  - Pattern selection with regex support
  - Visual selection indicators [✓] 
  - Batch operations on selected items

- **Enhanced file information**
  - Display file permissions in selection mode
  - Show owner and group information
  - Symlink detection and visual indicator (🔗)
  - Improved permission string display (rwxrwxrwx format)

- **Root mode features**
  - Automatic detection of root privileges
  - Additional keyboard shortcuts (s: select, p: pattern, c: chmod)
  - Root mode indicator in header
  - Extended controls in footer

### Changed
- Refactored code structure with modular design
- Improved error handling for permission operations
- Enhanced UI with box-drawing characters
- Better color coding for different file types
- More informative status messages

### Fixed
- Borrow checker issue with parent directory navigation
- Proper handling of symlinks
- Better permission preservation when using chmod

## [0.1.0] - 2025-01-XX

### Initial Release
- Basic terminal file system navigation
- Keyboard controls (arrows, Enter, Backspace, Esc/q)
- Directory breadcrumb display
- Visual indicators for files and folders
- Cross-platform support (Windows/macOS/Linux)
- Smooth scrolling for large directories
- Permission error handling
- Clean, minimalist interface
