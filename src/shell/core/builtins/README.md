# Built-in Commands Module

This module contains the native Rust implementations of various built-in shell commands. These commands are executed directly within the `my_cli_tool` application, providing cross-platform consistency and often better performance compared to spawning external processes.

## Purpose

To encapsulate the logic for commands that are fundamental to the shell's operation and are implemented directly in Rust. This separation improves modularity and allows for easier management and extension of built-in functionalities.

## Current State

The following built-in commands are currently implemented:

*   `alias`: Creates, displays, or removes command aliases. Supports `alias name=value` to create, `alias` to list all, and `unalias name` to remove.
*   `cat`: Concatenates and displays file contents.
*   `cd`: Changes the current working directory.
*   `cp`: Copies files.
*   `echo`: Displays a line of text.
*   `grep`: Searches for patterns in text.
*   `ls`: Lists the contents of a directory, with support for `-l` (long listing) and `-a` (all files) flags.
*   `mkdir`: Creates new directories.
*   `mv`: Moves (renames) files and directories.
*   `open`: Opens files and directories with their default applications.
*   `ping`: Sends ICMP echo requests to network hosts.
*   `rm`: Removes files and directories.

## To-Dos

*   **Error Handling Refinement:** Improve error messages and handling for built-in commands to provide more user-friendly feedback.
*   **Feature Expansion:** Consider adding more complex features to existing commands, such as recursive copy for `cp`.