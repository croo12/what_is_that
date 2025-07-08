# Shell Core Module

This module contains the core logic for our custom shell. It handles parsing commands, executing built-in commands (implemented natively in Rust), and falling back to external OS commands when a built-in is not found.

## Purpose

To provide a cross-platform, GUI-based command-line experience by implementing shell functionalities directly in Rust, rather than relying on external shell processes for all commands. This allows for greater control and consistency across different operating systems.

## Components

-   **`command_executor.rs`:** The main entry point for executing commands. It uses the `shlex` crate to parse the input string and then dispatches to built-in implementations or external OS commands.
-   **`autocompletion.rs`:** Provides context-aware command and path suggestions. This feature is now enabled and uses the `shlex` parser for more accurate, context-aware suggestions, including for paths with spaces.
-   **`builtins/`:** Native Rust implementations of common shell commands (e.g., `ls`, `cd`, `echo`).
-   **`external.rs`:** Logic for spawning and managing external processes when a command is not a built-in.

## Built-in Commands Implemented:

-   `ls`: Lists the contents of a directory.
-   `cd`: Changes the current working directory.
-   `ping`: Sends ICMP echo requests to network hosts.
-   `open`: Opens files and directories with their default applications.

## Dependencies

This module relies on `tokio` for asynchronous operations, `encoding_rs` for character encoding handling, and `shlex` for command parsing.

## Current State

The `shell_core` module has been refactored into smaller, more manageable sub-modules for improved organization and maintainability. The command executor now uses `shlex` to robustly parse user input, correctly handling quoted arguments. The autocompletion feature has been re-enabled and updated to use this new parser, providing a more reliable and intuitive user experience.