//! This module defines the main GUI application structure `GuiApp`.

use eframe::egui;
use tokio::sync::oneshot;

use super::tab::ShellTab;
use super::tab_bar;

/// `GuiApp` is the main structure for the GUI application.
/// It manages multiple shell tabs.
pub struct GuiApp {
    pub(super) tabs: Vec<ShellTab>,
    pub(super) selected_tab: usize,
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
        // Render the tab bar
        tab_bar::show(ctx, self);

        // Render the central panel for the selected tab
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
