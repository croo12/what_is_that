//! This module defines the graphical user interface (GUI) for the `my_cli_tool` application.
//! It uses the `eframe` and `egui` crates to create an interactive terminal-like experience.

mod tab;

use eframe::egui;
use tokio::sync::oneshot;

use tab::ShellTab;

/// `GuiApp` is the main structure for the GUI application.
/// It manages multiple shell tabs.
pub struct GuiApp {
    tabs: Vec<ShellTab>,
    selected_tab: usize,
    shutdown_sender: Option<oneshot::Sender<()>>,
}

impl GuiApp {
    /// Creates a new `GuiApp` instance.
    pub fn new(shutdown_sender: oneshot::Sender<()>) -> Self {
        Self {
            tabs: vec![ShellTab::new("Tab 1".to_string())],
            selected_tab: 0,
            shutdown_sender: Some(shutdown_sender),
        }
    }
}

impl eframe::App for GuiApp {
    /// Called once per frame to update the GUI.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("tabs").show(ctx, |ui| {
            ui.horizontal(|ui| {
                for (i, tab) in self.tabs.iter().enumerate() {
                    if ui.selectable_label(self.selected_tab == i, &tab.title).clicked() {
                        self.selected_tab = i;
                    }
                }
                if ui.button("+").clicked() {
                    let new_tab_index = self.tabs.len();
                    self.tabs.push(ShellTab::new(format!("Tab {}", new_tab_index + 1)));
                    self.selected_tab = new_tab_index;
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(tab) = self.tabs.get_mut(self.selected_tab) {
                tab.ui(ui);
            }
        });
    }

    /// Called when the application is about to exit.
    /// Sends a shutdown signal to background tasks.
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Some(sender) = self.shutdown_sender.take() {
            let _ = sender.send(());
        }
    }
}