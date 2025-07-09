# Changelog

All notable changes to this module will be documented in this file.

## [v0.2.0] - 2025-07-09

### Added
- **Pipeline (`|`) and Redirection (`>`) Support:** Implemented the core logic to parse and execute command pipelines and output redirection.
- **Built-in `grep` command:** Added a simple, cross-platform `grep` command that can be used in pipelines to filter text.

### Changed
- **Command Parser:** Replaced the simple `shlex`-based parser with a more robust implementation that understands shell operators (`|`, `>`).
- **`echo` command:** The built-in `echo` now correctly interprets common escape sequences such as `\n` and `\t`.
- **Error Handling:** Improved error messaging for non-existent commands.

