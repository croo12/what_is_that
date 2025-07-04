//! This module defines the state and UI for a single shell tab in the `my_cli_tool` application.

use eframe::egui;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task;
use std::path::PathBuf;
use chrono::Local;

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
    suggestions: Arc<Mutex<Vec<String>>>, 
    active_suggestion_index: Arc<Mutex<Option<usize>>>,
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
            suggestions: Arc::new(Mutex::new(Vec::new())),
            active_suggestion_index: Arc::new(Mutex::new(None)),
        }
    }

    /// Renders the UI for this tab.
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        // Asynchronously update current_dir_display
        let shell_core_arc_clone = self.shell_core.clone();
        let current_dir_display_arc_clone_for_spawn = self.current_dir_display.clone();
        task::spawn(async move {
            let shell_core = shell_core_arc_clone.lock().await;
            let new_dir = shell_core.get_current_dir().display().to_string();
            *current_dir_display_arc_clone_for_spawn.lock().await = new_dir;
        });

        ui.label(format!("Current Directory: {}", self.current_dir_display.try_lock().map(|s| s.clone()).unwrap_or_else(|_|"(Loading...)".to_string())));

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        // Scrollable area for displaying command output
        egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui_scroll| {
            let output_str = self.output.try_lock().map(|s| s.clone()).unwrap_or_else(|_|"(Output busy...)".to_string());
            ui_scroll.label(output_str);
        });

        let mut input_has_focus = false; // Declare here

        ui.horizontal(|ui| {
            ui.label("Command:");
            let response = ui.text_edit_singleline(&mut self.input);

            input_has_focus = ui.memory(|mem| mem.has_focus(response.id)); // Initialize here

            if response.changed() {
                // Generate suggestions when input changes
                let input_clone = self.input.clone();
                let autocompleter_clone = self.autocompleter.clone();
                let current_dir_display_arc_clone_for_suggestions = self.current_dir_display.clone();
                let suggestions_arc_clone = self.suggestions.clone();
                let active_suggestion_index_arc_clone = self.active_suggestion_index.clone();

                task::spawn(async move {
                    let current_dir = current_dir_display_arc_clone_for_suggestions.lock().await.clone();
                    let current_dir_path = PathBuf::from(current_dir);
                    let new_suggestions = autocompleter_clone.get_suggestions(&input_clone, &current_dir_path).await;
                    *suggestions_arc_clone.lock().await = new_suggestions;
                    *active_suggestion_index_arc_clone.lock().await = None;
                });
            }

            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                let selected_suggestion = {
                    let active_suggestion_index_guard = self.active_suggestion_index.try_lock();
                    let suggestions_guard = self.suggestions.try_lock();

                    if let (Ok(active_suggestion_index), Ok(suggestions)) = (active_suggestion_index_guard, suggestions_guard) {
                        if let Some(index) = *active_suggestion_index {
                            suggestions.get(index).map(|s| s.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                };
                if let Some(suggestion) = selected_suggestion {
                    self.input = suggestion;
                }
                self.execute_command();
                response.request_focus();
            }

            if input_has_focus {
                ui.input(|i| {
                    if i.key_pressed(egui::Key::ArrowUp) {
                        let mut active_suggestion_index = self.active_suggestion_index.try_lock().unwrap();
                        let suggestions = self.suggestions.try_lock().unwrap();
                        if !suggestions.is_empty() {
                            *active_suggestion_index = Some(match *active_suggestion_index {
                                Some(index) => if index > 0 { index - 1 } else { suggestions.len() - 1 },
                                None => suggestions.len() - 1,
                            });
                        } else if let Some(cmd) = self.command_history.navigate_up() {
                            self.input = cmd.to_owned();
                        }
                    } else if i.key_pressed(egui::Key::ArrowDown) {
                        let mut active_suggestion_index = self.active_suggestion_index.try_lock().unwrap();
                        let suggestions = self.suggestions.try_lock().unwrap();
                        if !suggestions.is_empty() {
                            *active_suggestion_index = Some(match *active_suggestion_index {
                                Some(index) => if index < suggestions.len() - 1 { index + 1 } else { 0 },
                                None => 0,
                            });
                        } else if let Some(cmd) = self.command_history.navigate_down() {
                            self.input = cmd.to_owned();
                        }
                    } else if i.key_pressed(egui::Key::Tab) {
                        let mut active_suggestion_index = self.active_suggestion_index.try_lock().unwrap();
                        let suggestions = self.suggestions.try_lock().unwrap();
                        if let Some(index) = *active_suggestion_index {
                            if let Some(suggestion) = suggestions.get(index) {
                                self.input = suggestion.clone();
                                self.suggestions.try_lock().unwrap().clear(); // Clear suggestions after selection
                                *active_suggestion_index = None;
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
        let suggestions_guard = self.suggestions.try_lock();
        let active_suggestion_index_guard = self.active_suggestion_index.try_lock();

        if let (Ok(suggestions), Ok(active_suggestion_index)) = (suggestions_guard, active_suggestion_index_guard) {
            if !suggestions.is_empty() && self.input.len() > 0 && input_has_focus {
                ui.group(|ui| {
                    ui.set_width(ui.available_width());
                    for (i, suggestion) in suggestions.iter().enumerate() {
                        let is_active = *active_suggestion_index == Some(i);
                        if ui.selectable_label(is_active, suggestion).clicked() {
                            self.input = suggestion.clone();
                            should_clear_suggestions = true;
                            // active_suggestion_index = None; // This needs to be set after the loop
                        }
                    }
                });
            }
        }
        
        if should_clear_suggestions {
            self.suggestions.try_lock().unwrap().clear();
            *self.active_suggestion_index.try_lock().unwrap() = None;
        }
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
                let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                output.push_str(&format!("\n[{}] {} $ {}\n", timestamp, current_dir, &input_command));
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
        self.suggestions.try_lock().unwrap().clear(); // Clear suggestions after command execution
        *self.active_suggestion_index.try_lock().unwrap() = None;
    }
}