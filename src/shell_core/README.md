# Shell Core Module

This module contains the core logic for our custom shell. It handles parsing commands, executing built-in commands (implemented natively in Rust), and falling back to external OS commands when a built-in is not found.

## Purpose

To provide a cross-platform, GUI-based command-line experience by implementing shell functionalities directly in Rust, rather than relying on external shell processes for all commands. This allows for greater control and consistency across different operating systems.

## Components

-   **`execute_shell_command` function:** The main entry point for executing commands. It dispatches to built-in implementations or external OS commands.
-   **Built-in Commands:** Native Rust implementations of common shell commands (e.g., `ls`, `cd`, `echo`).
-   **External Command Execution:** Logic for spawning and managing external processes when a command is not a built-in.
-   **Autocompletion:** Provides enhanced suggestions for commands and arguments, including support for complex path scenarios and arguments with spaces.

## Built-in Commands Implemented:

-   `ls`: Lists the contents of a directory.
-   `cd`: Changes the current working directory.
-   `ping`: Sends ICMP echo requests to network hosts.
-   `open`: Opens files and directories with their default applications.

## Dependencies

This module relies on `tokio` for asynchronous operations and `encoding_rs` for character encoding handling.

## Current State

The `shell_core` module has been refactored into smaller, more manageable sub-modules for improved organization and maintainability. It now includes:

*   **`builtins/`:** Contains individual implementations of built-in commands (`cd.rs`, `ls.rs`, `ping.rs`, `open.rs`).
*   **`command_executor.rs`:** Encapsulates the logic for dispatching commands to either built-in implementations or external system commands.
*   **`autocompletion.rs`:** Provides context-aware command and path suggestions.
    **Note:** The current implementation of autocompletion is temporarily disabled. It requires a more robust parsing engine to handle complex cases correctly. The future plan is to build a proper command-line parser first, and then re-implement the autocompletion feature on top of that solid foundation. This will ensure better accuracy and extensibility, similar to modern shells like zsh or PowerShell.