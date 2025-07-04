//! This module defines the state and UI for a single shell tab in the `my_cli_tool` application.

use eframe::egui;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task;
use std::path::PathBuf;

use crate::command_history::CommandHistory;
use crate::shell_core::ShellCore;
use crate::shell_core::autocompletion::Autocompleter;

/// `ShellTab` holds the state for a single tab, including input, output, and shell core.
pub struct ShellTab {
    pub title: String,
    input: String,
    output: Arc<Mutex<String>>,
    shell_core: Arc<Mutex<ShellCore>>,
    command_history: CommandHistory,
    current_dir_display: Arc<Mutex<String>>,
    autocompleter: Autocompleter,
    suggestions: Vec<String>,
    active_suggestion_index: Option<usize>,
}

impl ShellTab {
    /// Creates a new `ShellTab` instance.
    pub fn new(title: String) -> Self {
        let shell_core = Arc::new(Mutex::new(ShellCore::new()));
        let command_history = CommandHistory::new();
        let autocompleter = Autocompleter::new(command_history.clone());
        let current_dir = "Loading...".to_string();

        Self {
            title,
            input: String::new(),
            output: Arc::new(Mutex::new(String::new())),
            shell_core,
            command_history,
            current_dir_display: Arc::new(Mutex::new(current_dir)),
            autocompleter,
            suggestions: Vec::new(),
            active_suggestion_index: None,
        }
    }

    /// Renders the UI for this tab.
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        // Asynchronously update current_dir_display
        let shell_core_arc_clone = self.shell_core.clone();
        let current_dir_display_arc_clone = self.current_dir_display.clone();
        task::spawn(async move {
            let shell_core = shell_core_arc_clone.lock().await;
            let new_dir = shell_core.get_current_dir().display().to_string();
            *current_dir_display_arc_clone.lock().await = new_dir;
        });

        ui.label(format!("Current Directory: {}", self.current_dir_display.try_lock().map(|s| s.clone()).unwrap_or_else(|_| "(Loading...)".to_string())));

        ui.horizontal(|ui| {
            ui.label("Command:");
            let response = ui.text_edit_singleline(&mut self.input);

            if response.changed() {
                // Generate suggestions when input changes
                let current_dir = self.current_dir_display.try_lock().map(|s| PathBuf::from(s.clone())).unwrap_or_else(|_| PathBuf::from("."));
                self.suggestions = self.autocompleter.get_suggestions(&self.input, &current_dir);
                self.active_suggestion_index = None;
            }

            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                if let Some(index) = self.active_suggestion_index {
                    if let Some(suggestion) = self.suggestions.get(index) {
                        self.input = suggestion.clone();
                    }
                }
                self.execute_command();
                response.request_focus();
            }

            if response.has_focus() {
                ui.input(|i| {
                    if i.key_pressed(egui::Key::ArrowUp) {
                        if !self.suggestions.is_empty() {
                            self.active_suggestion_index = Some(match self.active_suggestion_index {
                                Some(index) => if index > 0 { index - 1 } else { self.suggestions.len() - 1 },
                                None => self.suggestions.len() - 1,
                            });
                        } else if let Some(cmd) = self.command_history.navigate_up() {
                            self.input = cmd.to_owned();
                        }
                    } else if i.key_pressed(egui::Key::ArrowDown) {
                        if !self.suggestions.is_empty() {
                            self.active_suggestion_index = Some(match self.active_suggestion_index {
                                Some(index) => if index < self.suggestions.len() - 1 { index + 1 } else { 0 },
                                None => 0,
                            });
                        } else if let Some(cmd) = self.command_history.navigate_down() {
                            self.input = cmd.to_owned();
                        }
                    } else if i.key_pressed(egui::Key::Tab) {
                        if let Some(index) = self.active_suggestion_index {
                            if let Some(suggestion) = self.suggestions.get(index) {
                                self.input = suggestion.clone();
                                self.suggestions.clear(); // Clear suggestions after selection
                                self.active_suggestion_index = None;
                            }
                        }
                    }
                });
            }

            if ui.button("Run").clicked() {
                self.execute_command();
            }

            if ui.button("Clear").clicked() {
                let output_arc = self.output.clone();
                task::spawn(async move {
                    output_arc.lock().await.clear();
                });
            }
        });

        // Display suggestions
        let mut should_clear_suggestions = false;
        if !self.suggestions.is_empty() && self.input.len() > 0 {
            ui.group(|ui| {
                ui.set_width(ui.available_width());
                for (i, suggestion) in self.suggestions.iter().enumerate() {
                    let is_active = self.active_suggestion_index == Some(i);
                    if ui.selectable_label(is_active, suggestion).clicked() {
                        self.input = suggestion.clone();
                        should_clear_suggestions = true;
                        self.active_suggestion_index = None;
                    }
                }
            });
        }
        if should_clear_suggestions {
            self.suggestions.clear();
        }

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui_scroll| {
            let output_str = self.output.try_lock().map(|s| s.clone()).unwrap_or_else(|_| "(Output busy...)".to_string());
            ui_scroll.label(output_str);
        });
    }

    /// Executes the command currently in the input field.
    fn execute_command(&mut self) {
        let input_command = self.input.trim().to_string();
        if input_command.is_empty() {
            return;
        }

        self.command_history.add(input_command.clone());

        let output_arc = self.output.clone();
        let shell_core_arc = self.shell_core.clone();
        let current_dir_display_arc = self.current_dir_display.clone();

        task::spawn(async move {
            {
                let mut output = output_arc.lock().await;
                let current_dir = current_dir_display_arc.lock().await;
                output.push_str(&format!("[{}] $ {}\n", current_dir, &input_command));
            }

            let command_output = {
                let mut shell_core = shell_core_arc.lock().await;
                shell_core.execute_shell_command(&input_command).await
            };

            {
                let mut output = output_arc.lock().await;
                output.push_str(&command_output);
                output.push('\n');
            }

            {
                let shell_core = shell_core_arc.lock().await;
                let new_dir = shell_core.get_current_dir().display().to_string();
                *current_dir_display_arc.lock().await = new_dir;
            }
        });

        self.input.clear();
        self.suggestions.clear(); // Clear suggestions after command execution
        self.active_suggestion_index = None;
    }
}