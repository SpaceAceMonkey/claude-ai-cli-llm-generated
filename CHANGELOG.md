# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [4.0.0] - 2024-12-XX

### ⚠️ KNOWN ISSUE
**WARNING: This version contains a critical bug that causes the application to crash during window resizing, especially when resizing rapidly. Users are advised to avoid rapid terminal resizing or consider using a previous version until this issue is resolved.**

### Added
- Color configuration system with customizable themes
- Color profile management with predefined themes
- Enhanced dialog system for configuration
- Cross-platform keyboard shortcuts for better terminal compatibility
- Multiple keyboard shortcuts for color configuration dialog for maximum terminal compatibility:
  - `Ctrl+Shift+C` (primary reliable shortcut)
  - `F3` function key (very reliable across terminals)
  - `Ctrl+Alt+C` (more reliable than Alt+Shift)
  - `Alt+Shift+C` (legacy support, kept for backwards compatibility)
- Multiple keyboard shortcuts for color profile dialog for maximum terminal compatibility:
  - `Ctrl+Shift+P` (primary reliable shortcut)
  - `F4` function key (very reliable across terminals)
  - `Ctrl+Alt+P` (more reliable than Alt+Shift)
  - `Alt+Shift+P` (legacy support, kept for backwards compatibility)

### Changed
- Improved user interface styling and layout
- Enhanced dialog presentation and interaction
- Enhanced styling for in-app dialogs to make them stand out more against the main UI
- Better visual distinction between dialogs and the main application interface
- Improved color and color profile dialog user experience

### Fixed
- Various UI rendering improvements
- Better error handling in configuration dialogs
- Enhanced terminal compatibility for color configuration shortcuts
- Better cross-platform support for modifier key combinations

## [3.0.0] - 2024-11-XX

### Added
- File save and load functionality
- Directory navigation in file dialogs
- Enhanced keyboard shortcuts for file operations

### Changed
- Improved UI layout and responsiveness
- Better error handling and user feedback

### Fixed
- Terminal compatibility issues
- File dialog navigation improvements

## [2.0.0] - 2024-10-XX

### Added
- Interactive TUI (Terminal User Interface)
- Real-time chat interface
- Keyboard navigation and shortcuts

### Changed
- Migrated from command-line only to full TUI
- Enhanced user experience with visual interface

### Fixed
- Input handling improvements
- Display rendering optimizations

## [1.0.0] - 2024-09-XX

### Added
- Initial release
- Basic Claude AI API integration
- Command-line interface
- Message sending and receiving functionality
- Configuration support for API keys and models