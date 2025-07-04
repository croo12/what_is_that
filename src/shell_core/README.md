# Shell Core Module

This module contains the core logic for our custom shell. It handles parsing commands, executing built-in commands (implemented natively in Rust), and falling back to external OS commands when a built-in is not found.

## Purpose

To provide a cross-platform, GUI-based command-line experience by implementing shell functionalities directly in Rust, rather than relying on external shell processes for all commands. This allows for greater control and consistency across different operating systems.

## Components

-   **`execute_shell_command` function:** The main entry point for executing commands. It dispatches to built-in implementations or external OS commands.
-   **Built-in Commands:** Native Rust implementations of common shell commands (e.g., `ls`, `cd`, `echo`).
-   **External Command Execution:** Logic for spawning and managing external processes when a command is not a built-in.
-   **Autocompletion:** Provides suggestions for commands and arguments based on history and built-in commands.

## Built-in Commands Implemented:

-   `ls`: Lists the contents of a directory.
-   `cd`: Changes the current working directory.
-   `ping`: Sends ICMP echo requests to network hosts.
-   `open`: Opens files and directories with their default applications.

## Dependencies

This module relies on `tokio` for asynchronous operations and `encoding_rs` for character encoding handling.