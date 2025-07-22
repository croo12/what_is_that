//! This module defines the state and UI for a single shell tab in the `my_cli_tool` application.

use eframe::egui;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task;
use chrono::Local;

use crate::shell::history::CommandHistory;
use crate::shell::core::ShellCore;
use crate::shell::features::autocompletion::Autocompleter;

/// `ShellTab` holds the state for a single tab, including input, output, and shell core.
pub struct ShellTab {
    pub title: String,
    input: String,
    output: Arc<Mutex<String>>,
    shell_core: Arc<Mutex<ShellCore>>,
    command_history: CommandHistory,
    current_dir_display: Arc<Mutex<String>>,
    git_info_display: Arc<Mutex<String>>,
    autocompleter: Autocompleter,
    ghost_text: Arc<Mutex<String>>,
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
            git_info_display: Arc::new(Mutex::new(String::new())),
            autocompleter,
            ghost_text: Arc::new(Mutex::new(String::new())),
        }
    }

    /// Renders the UI for this tab.
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        // Asynchronously update current_dir_display and git_info_display
        let shell_core_arc_clone = self.shell_core.clone();
        let current_dir_display_arc_clone_for_spawn = self.current_dir_display.clone();
        let git_info_display_arc_clone_for_spawn = self.git_info_display.clone();
        task::spawn(async move {
            let shell_core = shell_core_arc_clone.lock().await;
            let new_dir = shell_core.get_current_dir().to_string_lossy().into_owned();
            *current_dir_display_arc_clone_for_spawn.lock().await = new_dir;

            let git_info_str = if let Some(info) = &shell_core.git_info {
                let changes_indicator = if info.has_changes { "*" } else { "" };
                format!("({}{})", info.branch_name, changes_indicator)
            } else {
                String::new()
            };
            *git_info_display_arc_clone_for_spawn.lock().await = git_info_str;
        });

        // Handle Tab key press for autocompletion BEFORE the main UI panel
        if ui.input(|i| i.key_pressed(egui::Key::Tab)) {
            if let Ok(ghost_text) = self.ghost_text.try_lock() {
                if !ghost_text.is_empty() && ghost_text.starts_with(&self.input) {
                    self.input = ghost_text.clone();
                    // Consume the Tab key event so it doesn't trigger other behaviors
                    ui.ctx().input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Tab));
                }
            }
        }

        let mut input_id = None;

        // Bottom panel for command input
        egui::TopBottomPanel::bottom("input_panel").show(ui.ctx(), |ui| {
            ui.horizontal(|ui| {
                ui.label("Command:");

                let mut layouter = |ui: &egui::Ui, string: &str, _wrap_width: f32| {
                    let mut layout_job = egui::text::LayoutJob::default();
                    let default_text_format = egui::TextFormat {
                        color: ui.style().visuals.text_color(),
                        ..Default::default()
                    };

                    if let Ok(ghost_text) = self.ghost_text.try_lock() {
                        if !ghost_text.is_empty() && ghost_text.starts_with(string) && !string.is_empty() {
                            // User-typed part
                            layout_job.append(string, 0.0, default_text_format.clone());
                            // Ghost text part
                            let suggestion_part = &ghost_text[string.len()..];
                            layout_job.append(
                                suggestion_part,
                                0.0,
                                egui::TextFormat {
                                    color: ui.style().visuals.weak_text_color(),
                                    ..Default::default()
                                },
                            );
                        } else {
                            layout_job.append(string, 0.0, default_text_format);
                        }
                    } else {
                         layout_job.append(string, 0.0, default_text_format);
                    }
                    ui.fonts(|f| f.layout_job(layout_job))
                };

                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.input)
                        .desired_width(f32::INFINITY)
                        .layouter(&mut layouter)
                );
                input_id = Some(response.id);

                if response.changed() {
                    let input_clone = self.input.clone();
                    let autocompleter_clone = self.autocompleter.clone();
                    let shell_core_clone = self.shell_core.clone();
                    let ghost_text_clone = self.ghost_text.clone();

                    task::spawn(async move {
                        let shell_core = shell_core_clone.lock().await;
                        let suggestions = autocompleter_clone.get_suggestions(&input_clone, &shell_core.get_current_dir()).await;
                        let mut ghost_text = ghost_text_clone.lock().await;
                        if let Some(first_suggestion) = suggestions.get(0) {
                            *ghost_text = first_suggestion.clone();
                        } else {
                            ghost_text.clear();
                        }
                    });
                }

                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.execute_command();
                    response.request_focus();
                }

                if ui.button("Run").clicked() {
                    self.execute_command();
                }

                if ui.button("Clear").clicked() {
                    let output_arc = self.output.clone();
                    tokio::task::spawn(async move {
                        output_arc.lock().await.clear();
                    });
                }
            });
        });

        // Central panel for output
        egui::CentralPanel::default().show(ui.ctx(), |ui| {
            let dir_str = self.current_dir_display.try_lock().map(|s| s.clone()).unwrap_or_else(|_|"(Loading...)".to_string());
            let git_str = self.git_info_display.try_lock().map(|s| s.clone()).unwrap_or_default();
            ui.label(format!("Current Directory: {} {}", dir_str, git_str));
            ui.separator();

            egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui_scroll| {
                let output_str = self.output.try_lock().map(|s| s.clone()).unwrap_or_else(|_|"(Output busy...)".to_string());
                ui_scroll.set_width(ui_scroll.available_width());
                ui_scroll.add(egui::Label::new(egui::RichText::new(&output_str).monospace()).wrap(true));
            });
        });

        if let Some(id) = input_id {
            if ui.memory(|mem| mem.has_focus(id)) {
                ui.input(|i| {
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
        let git_info_display_arc = self.git_info_display.clone();

        task::spawn(async move {
            {
                let mut output = output_arc.lock().await;
                let current_dir = current_dir_display_arc.lock().await;
                let git_info = git_info_display_arc.lock().await;
                let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                output.push_str(&format!("\n[{}] {} {} $ {}\n", timestamp, *current_dir, *git_info, &input_command));
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
                // The git info is already updated inside execute_shell_command
                let new_dir = shell_core.get_current_dir().to_string_lossy().into_owned();
                *current_dir_display_arc.lock().await = new_dir;
                
                let git_info_str = if let Some(info) = &shell_core.git_info {
                    let changes_indicator = if info.has_changes { "*" } else { "" };
                    format!("({}{})", info.branch_name, changes_indicator)
                } else {
                    String::new()
                };
                *git_info_display_arc.lock().await = git_info_str;
            }
        });

        self.input.clear();
        // Clear ghost text after command execution
        let ghost_text_clone = self.ghost_text.clone();
        task::spawn(async move {
            ghost_text_clone.lock().await.clear();
        });
    }
}