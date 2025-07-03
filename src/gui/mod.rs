//! This module defines the graphical user interface (GUI) for the `my_cli_tool` application.
//! It uses the `eframe` and `egui` crates to create an interactive terminal-like experience.

use eframe::egui;
use std::sync::Arc;
use tokio::sync::{Mutex};
use tokio::task;
use tokio::sync::oneshot;

use crate::command_history::CommandHistory;
use crate::shell_core::ShellCore;

/// `TemplateApp` is the main structure for the GUI application.
/// It holds the state of the input, output, command history, and the shell core.
pub struct TemplateApp {
    /// The current text in the input field.
    input: String,
    /// The output displayed in the terminal area, shared across tasks.
    output: Arc<Mutex<String>>,
    /// The core shell logic, shared across tasks.
    shell_core: Arc<Mutex<ShellCore>>,
    /// Manages the history of executed commands.
    command_history: CommandHistory,
    /// Sender for the shutdown signal to background tasks.
    shutdown_sender: Option<oneshot::Sender<()>>,
    /// The current working directory to be displayed in the GUI.
    current_dir_display: Arc<Mutex<String>>,
}

impl TemplateApp {
    /// Creates a new `TemplateApp` instance.
    ///
    /// # Arguments
    ///
    /// * `output_arc` - An `Arc<Mutex<String>>` for sharing the output string.
    /// * `shell_core_arc` - An `Arc<Mutex<ShellCore>>` for sharing the shell core instance.
    /// * `shutdown_sender` - A `oneshot::Sender` to signal shutdown to background tasks.
    pub fn new(output_arc: Arc<Mutex<String>>, shell_core_arc: Arc<Mutex<ShellCore>>, shutdown_sender: oneshot::Sender<()>) -> Self {
        Self {
            input: String::new(),
            output: output_arc,
            shell_core: shell_core_arc,
            command_history: CommandHistory::new(),
            shutdown_sender: Some(shutdown_sender),
            current_dir_display: Arc::new(Mutex::new("Loading...".to_string())),
        }
    }
}

impl eframe::App for TemplateApp {
    /// Called once per frame to update the GUI.
    ///
    /// This method handles user input, executes commands, updates the output display,
    /// and manages history navigation.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The `egui::Context` for the current frame.
    /// * `_frame` - The `eframe::Frame` for the current frame (unused).
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui|
        {
            ui.heading("my_cli_tool GUI");
            ui.label(format!("Current Directory: {}", match self.current_dir_display.try_lock() {
                Ok(guard) => guard.clone(),
                Err(_) => "(Loading...)".to_string(),
            }));

            // Asynchronously update current_dir_display
            let shell_core_arc_clone = self.shell_core.clone();
            let current_dir_display_arc_clone = self.current_dir_display.clone();
            task::spawn(async move {
                let shell_core = shell_core_arc_clone.lock().await;
                let new_dir = shell_core.get_current_dir().display().to_string();
                *current_dir_display_arc_clone.lock().await = new_dir;
            });

            ui.horizontal(|ui|
            {
                ui.label("Command:");
                let response = ui.text_edit_singleline(&mut self.input);

                if response.lost_focus() && ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                    // Execute command on Enter key press
                    let input_command = self.input.clone();
                    if !input_command.is_empty() {
                        self.command_history.add(input_command.clone());
                    }

                    let output_arc = self.output.clone();
                    let shell_core_arc = self.shell_core.clone();
                    let current_dir_display_arc = self.current_dir_display.clone();
                    // Spawn a new task to execute the command asynchronously.
                    // This prevents blocking the UI thread.
                    task::spawn(async move {
                        let mut shell_core = shell_core_arc.lock().await;
                        let command_output: String = shell_core.execute_shell_command(&input_command).await;
                        *output_arc.lock().await = format!("{}{}\n", current_dir_display_arc.lock().await.clone(), command_output);
                        *current_dir_display_arc.lock().await = shell_core.get_current_dir().display().to_string();
                    });
                    self.input.clear(); // Clear immediately after spawning task
                }

                // History navigation
                // Handles ArrowUp and ArrowDown for navigating command history.
                if response.has_focus() {
                    ctx.input(|i| {
                        if i.key_pressed(egui::Key::ArrowUp) {
                            if let Some(cmd) = self.command_history.navigate_up() {
                                self.input = cmd.to_owned();
                            }
                        } else if i.key_pressed(egui::Key::ArrowDown) {
                            if let Some(cmd) = self.command_history.navigate_down() {
                                self.input = cmd.to_owned();
                            }
                        }
                    });
                }

                // Run button click handler
                if ui.button("Run").clicked() {
                    let input_command = self.input.clone();
                    if !input_command.is_empty() {
                        self.command_history.add(input_command.clone());
                    }

                    let output_arc = self.output.clone();
                    let shell_core_arc = self.shell_core.clone();
                    let current_dir_display_arc = self.current_dir_display.clone();
                    // Spawn a new task to execute the command asynchronously.
                    // This prevents blocking the UI thread.
                    task::spawn(async move {
                        let mut shell_core = shell_core_arc.lock().await;
                        let command_output: String = shell_core.execute_shell_command(&input_command).await;
                        *output_arc.lock().await = format!("{}{}\n", current_dir_display_arc.lock().await.clone(), command_output);
                        *current_dir_display_arc.lock().await = shell_core.get_current_dir().display().to_string();
                    });
                    self.input.clear(); // Clear immediately after spawning task
                }

                // Clear button click handler
                if ui.button("Clear").clicked() {
                    // Use async lock for output as well
                    let output_arc = self.output.clone();
                    task::spawn(async move {
                        output_arc.lock().await.clear();
                    });
                }
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            // Scrollable area for displaying command output
            egui::ScrollArea::vertical().show(ui, |ui_scroll|
            {
                // Use try_lock for non-blocking access to output
                let output_str = match self.output.try_lock() {
                    Ok(guard) => guard.clone(),
                    Err(_) => "(Output busy...)".to_string(), // Display a message if locked
                };
                ui_scroll.label(output_str);
            });
        });
    }

    /// Called when the application is about to exit.
    /// Sends a shutdown signal to background tasks.
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // Send shutdown signal to the background task
        if let Some(sender) = self.shutdown_sender.take() {
            let _ = sender.send(());
        }
    }
}