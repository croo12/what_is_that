mod gui;
mod shell_core;
mod command_history;

use eframe::egui;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task;
use tokio::sync::oneshot;

fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // Install my own font (e.g. a Japanese font):
    fonts.font_data.insert(
        "korean_font".to_owned(),
        egui::FontData::from_static(include_bytes!("C:/Windows/Fonts/malgunbd.ttf")), // Malgun Gothic Bold
    );

    // Put my font first (highest priority):
    fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap().insert(0, "korean_font".to_owned());

    // Put my font as last resort for monospace fonts:
    fonts.families.get_mut(&egui::FontFamily::Monospace).unwrap().push("korean_font".to_owned());

    ctx.set_fonts(fonts);
}

#[tokio::main]
async fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    let output_arc = Arc::new(Mutex::new(String::new()));

    let output_arc_clone_for_task = output_arc.clone();

    let (tx, rx) = oneshot::channel(); // Channel for shutdown signal

    let app_result = eframe::run_native(
        "my_cli_tool GUI",
        options,
        Box::new(move |cc| {
            setup_fonts(&cc.egui_ctx);
            let egui_ctx_for_task = cc.egui_ctx.clone(); // Clone the actual context

            task::spawn(async move {
                let mut rx = rx; // Take ownership of the receiver
                loop {
                    tokio::select! {
                        _ = &mut rx => {
                            // Received shutdown signal
                            break;
                        }
                        _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                            // No longer reading from a persistent session, so no output to read here
                            // The output is updated directly by the command execution task
                        }
                    }
                }
            });

            Ok(Box::new(gui::TemplateApp::new(output_arc.clone(), tx)))
        }),
    );

    app_result
}
