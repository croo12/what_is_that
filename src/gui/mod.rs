use anyhow::Result;
use eframe::egui;
use std::sync::Arc;
use tokio::sync::{Mutex};
use tokio::task;
use tokio::sync::oneshot;

use crate::command_history::CommandHistory;
use crate::shell_core;

pub struct TemplateApp {
    input: String,
    output: Arc<Mutex<String>>,
    command_history: CommandHistory,
    shutdown_sender: Option<oneshot::Sender<()>>,
}

impl TemplateApp {
    pub fn new(output_arc: Arc<Mutex<String>>, shutdown_sender: oneshot::Sender<()>) -> Self {
        Self {
            input: String::new(),
            output: output_arc,
            command_history: CommandHistory::new(),
            shutdown_sender: Some(shutdown_sender),
        }
    }
}

impl eframe::App for TemplateApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui|
        {
            ui.heading("my_cli_tool GUI");

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
                    task::spawn(async move {
                        let command_output = shell_core::execute_shell_command(&input_command).await;
                        output_arc.lock().await.push_str(&command_output);
                    });
                    self.input.clear(); // Clear immediately after spawning task
                }

                // History navigation
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

                if ui.button("Run").clicked() {
                    let input_command = self.input.clone();
                    if !input_command.is_empty() {
                        self.command_history.add(input_command.clone());
                    }

                    let output_arc = self.output.clone();
                    task::spawn(async move {
                        let command_output = shell_core::execute_shell_command(&input_command).await;
                        output_arc.lock().await.push_str(&command_output);
                    });
                    self.input.clear(); // Clear immediately after spawning task
                }

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

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // Send shutdown signal to the background task
        if let Some(sender) = self.shutdown_sender.take() {
            let _ = sender.send(());
        }
    }
}