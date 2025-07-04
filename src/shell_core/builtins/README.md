# Built-in Commands Module

This module contains the native Rust implementations of various built-in shell commands. These commands are executed directly within the `my_cli_tool` application, providing cross-platform consistency and often better performance compared to spawning external processes.

## Purpose

To encapsulate the logic for commands that are fundamental to the shell's operation and are implemented directly in Rust. This separation improves modularity and allows for easier management and extension of built-in functionalities.

## Current State

The following built-in commands are currently implemented:

*   `cd`: Changes the current working directory.
*   `ls`: Lists the contents of a directory.
*   `ping`: Sends ICMP echo requests to network hosts.
*   `open`: Opens files and directories with their default applications.

## To-Dos

*   **Expand `ls` functionality:** Add support for common `ls` flags (e.g., `-l` for long listing, `-a` for all files).
*   **Implement `mkdir`:** Create a built-in command for creating new directories.
*   **Implement `rm`:** Create a built-in command for removing files and directories.
*   **Implement `cp` and `mv`:** Add built-in commands for copying and moving files/directories.
*   **Error Handling Refinement:** Improve error messages and handling for built-in commands to provide more user-friendly feedback.
