# my_cli_tool

This project is being transformed from a command-line interface (CLI) tool into a graphical user interface (GUI) application.

## Current State

Initially, `my_cli_tool` was a simple Rust CLI application that greeted a specified name. It was then modified to be a basic interactive shell, allowing users to execute commands and view their output directly in the terminal.

## New Direction: GUI Application

The project is now transitioning to a GUI application using the `egui` framework. This will provide a more user-friendly and visually interactive experience for executing commands.

## Planned Features

To achieve this, the following features will be implemented:

1.  **Cross-Platform GUI:** The application will utilize `egui` and `eframe` to create a native, cross-platform GUI.
2.  **Interactive Command Input:** A text input field within the GUI will allow users to type commands.
3.  **Output Display Area:** A dedicated area in the GUI will display the standard output and standard error of executed commands.
4.  **Asynchronous Command Execution:** Commands will be executed asynchronously to ensure the GUI remains responsive during long-running operations.
5.  **Modular Design:** The GUI logic will be encapsulated within a dedicated `src/gui` module for better organization and maintainability.

## Technical Approach

-   `egui` and `eframe` crates will be added as dependencies.
-   The `src/main.rs` will be refactored to initialize the `eframe` application.
-   A new module `src/gui/mod.rs` will be created to house all GUI-related code.
-   `std::process::Command` will still be used for external command execution, but integrated asynchronously with `tokio` within the GUI context.
-   `anyhow` will continue to be used for simplified error handling.