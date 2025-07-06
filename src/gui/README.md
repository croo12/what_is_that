# GUI Module

This module defines the graphical user interface (GUI) for the `my_cli_tool` application. It is built using the `egui` and `eframe` crates, providing a cross-platform and interactive shell experience.

## Purpose

To provide a user-friendly and visually interactive front-end for the underlying shell core. It handles user input, displays command output, and manages the overall visual presentation and user interaction flow.

## Current State

The GUI currently features:

*   **Multi-Tabbed Interface:** Allows users to manage multiple independent shell sessions simultaneously.
*   **Timestamped Command Output:** Displays command output with timestamps for improved readability and context.
*   **Command Input at Bottom:** The command input field is positioned at the bottom of the terminal area, mimicking traditional CLI layouts.
*   **Context-Aware Autocompletion Display:** Dynamically shows suggestions for commands and file paths as the user types, with keyboard navigation support.

## To-Dos

*   **Syntax Highlighting for Output:** Basic output formatting (errors, success) implemented.
*   **Clickable Links in Output:** Make URLs and file paths in the output clickable.
*   **Customizable Themes:** Allow users to select different color themes for the GUI.
*   **Resizable Panes:** Enable resizing of the input and output areas.
*   **Improved Scrollbar Experience:** Enhance the scrollbar for very long outputs.
*   **Settings/Preferences UI:** Add a dedicated UI for configuring application settings.