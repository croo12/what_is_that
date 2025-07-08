//! This module handles the rendering of the tab bar in the GUI.

use eframe::egui;
use super::app::GuiApp;
use super::tab::ShellTab;

/// Renders the tab bar and handles tab selection and creation.
pub fn show(ctx: &egui::Context, app: &mut GuiApp) {
    egui::TopBottomPanel::top("tabs").show(ctx, |ui| {
        ui.horizontal(|ui| {
            for (i, tab) in app.tabs.iter().enumerate() {
                if ui.selectable_label(app.selected_tab == i, &tab.title).clicked() {
                    app.selected_tab = i;
                }
            }
            if ui.button("+").clicked() {
                let new_tab_index = app.tabs.len();
                app.tabs.push(ShellTab::new(format!("Tab {}", new_tab_index + 1)));
                app.selected_tab = new_tab_index;
            }
        });
    });
}
