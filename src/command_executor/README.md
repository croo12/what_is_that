# Command Executor Module

This module is responsible for executing external commands and capturing their output.

## Purpose

To separate the command execution logic from the graphical user interface (GUI) concerns, promoting a cleaner architecture and better maintainability. This module provides a centralized function for running system commands, making it easier to modify or extend how commands are processed in the future.

## Components

-   **`execute_command` function:** A public function that takes a command string as input, executes it using PowerShell (on Windows), and returns the combined standard output and standard error as a `String`.

## Dependencies

This module primarily relies on Rust's standard library for process management (`std::process::Command`) and string manipulation.
