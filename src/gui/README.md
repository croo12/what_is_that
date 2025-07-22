# GUI Module

This module defines the graphical user interface (GUI) for the `my_cli_tool` application. It is built using the `egui` and `eframe` crates, providing a cross-platform and interactive shell experience.

## Purpose

To provide a user-friendly and visually interactive front-end for the underlying shell core. It handles user input, displays command output, and manages the overall visual presentation and user interaction flow.

## Current State

The GUI has been refactored into a more modular structure for better organization and maintainability:
*   **`app.rs`**: Contains the main `GuiApp` struct and the core application state.
*   **`tab.rs`**: Defines the UI and state for a single shell tab, **including its own `ShellCore` instance to manage shell-specific states like the current directory and command aliases.**
*   **`tab_bar.rs`**: Manages the rendering and interaction of the tab bar.

The GUI currently features:

*   **Multi-Tabbed Interface:** Allows users to manage multiple independent shell sessions simultaneously.
*   **Timestamped Command Output:** Displays command output with timestamps for improved readability and context.
*   **Command Input at Bottom:** The command input field is positioned at the bottom of the terminal area, mimicking traditional CLI layouts.
*   **Context-Aware Autocompletion Display:** Dynamically shows suggestions for commands and file paths as the user types, with keyboard navigation support.

## To-Dos

*   **File System Explorer UI:** Integrate a graphical file system browser for intuitive directory navigation and file operations.
*   **Customizable Themes & Fonts:** Allow users to select different color themes and font settings for the GUI.
*   **Settings/Preferences UI:** Add a dedicated UI for configuring application settings.
*   **Improved Output Formatting:** Enhance output readability with syntax highlighting or clickable elements.
