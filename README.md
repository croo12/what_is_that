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

## Module Structure

This project is organized into several modules, each responsible for a specific part of the application's functionality.

### `src/main.rs`
The main entry point of the application. It sets up the GUI framework, initializes shared application state, and manages the main event loop.
애플리케이션의 주요 진입점입니다. GUI 프레임워크를 설정하고, 공유 애플리케이션 상태를 초기화하며, GUI의 메인 이벤트 루프를 관리합니다.

### `src/gui/mod.rs`
Defines the graphical user interface (GUI) components and their interactions. This module handles user input, displays command output, and manages the overall visual presentation.
그래픽 사용자 인터페이스(GUI) 구성 요소 및 상호 작용을 정의합니다. 이 모듈은 사용자 입력을 처리하고, 명령 출력을 표시하며, 전반적인 시각적 표현을 관리합니다.

### `src/shell_core/mod.rs`
Provides the core shell functionality, including command parsing, execution, and management of the current working directory. It dispatches commands to specialized sub-modules.
명령 구문 분석, 실행 및 현재 작업 디렉토리 관리를 포함한 핵심 셸 기능을 제공합니다. 특수화된 하위 모듈로 명령을 전달합니다.

#### `src/shell_core/ls.rs`
Implements the built-in `ls` command for listing directory contents.
디렉토리 내용을 나열하는 내장 `ls` 명령을 구현합니다.

#### `src/shell_core/ping.rs`
Implements the built-in `ping` command for sending ICMP echo requests to network hosts.
네트워크 호스트에 ICMP 에코 요청을 보내는 내장 `ping` 명령을 구현합니다.

#### `src/shell_core/cd.rs`
Implements the built-in `cd` command for changing the current working directory.
현재 작업 디렉토리를 변경하는 내장 `cd` 명령을 구현합니다.

#### `src/shell_core/external.rs`
Handles the execution of external system commands that are not built into the shell.
셸에 내장되지 않은 외부 시스템 명령의 실행을 처리합니다.

### `src/command_history/mod.rs`
Manages the history of commands entered by the user, allowing for navigation and recall of previous commands.
사용자가 입력한 명령의 기록을 관리하여 이전 명령을 탐색하고 불러올 수 있도록 합니다.

## Technical Approach

-   `egui` and `eframe` crates will be added as dependencies.
-   The `src/main.rs` will be refactored to initialize the `eframe` application.
-   A new module `src/gui/mod.rs` will be created to house all GUI-related code.
-   `std::process::Command` will still be used for external command execution, but integrated asynchronously with `tokio` within the GUI context.
-   `anyhow` will continue to be used for simplified error handling.