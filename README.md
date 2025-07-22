# my_cli_tool

This project is a cross-platform graphical user interface (GUI) shell application built with Rust and `egui`.

## Project Goal

The primary goal of `my_cli_tool` is to provide a user-friendly and visually interactive command-line experience that feels like a native CLI. This involves leveraging underlying operating system functionalities directly where appropriate, enhancing performance, integration, and the overall 'native' feel of the shell.

## Current State

`my_cli_tool` currently provides a multi-tabbed interface for independent shell sessions, each with its own command input, timestamped output display, and current working directory. It features context-aware autocompletion for commands and file paths, and a native `open` command that utilizes OS-specific functionalities. **Additionally, it supports command piping (`|`) for chaining commands and includes built-in `alias` and `unalias` commands for creating and managing command shortcuts.**

## Module Structure

This project is organized into several modules, each responsible for a specific part of the application's functionality:

*   **`src/main.rs`:** The main entry point of the application. It sets up the GUI framework, initializes shared application state, and manages the main event loop.

*   **`src/gui/`:** Defines the graphical user interface (GUI) components and their interactions. (See `src/gui/README.md` for more details).

*   **`src/shell/core/`:** Provides the core shell functionality, including command parsing, execution, and management of the current working directory. (See `src/shell/core/README.md` for more details).

*   **`src/shell/history/`:** Manages the history of commands entered by the user, allowing for navigation and recall of previous commands.

*   **`src/shell/features/autocompletion/`:** Provides context-aware command and path autocompletion. (See `src/shell/features/autocompletion/README.md` for more details).

## Technical Approach

-   `egui` and `eframe` crates are used for the cross-platform GUI.
-   `tokio` is used for asynchronous command execution, ensuring a responsive UI.
-   `anyhow` is used for simplified error handling.
-   `shlex` is used for robust command-line argument parsing, especially for handling quoted arguments and pipelines.