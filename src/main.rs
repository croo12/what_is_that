//! This is the main entry point for the `my_cli_tool` graphical user interface (GUI) application.
//! It sets up the eframe application, initializes the shell core and command history,
//! and handles the main event loop for the GUI.

mod gui;
pub mod shell_core;
pub mod command_history;

use eframe::egui;
use tokio::sync::oneshot;
use tokio::task;

/// Sets up custom fonts for the egui context.
///
/// This function loads a Korean font (Malgun Gothic Bold) and sets it as the
/// primary proportional font and a fallback for monospace fonts.
///
/// # Arguments
///
/// * `ctx` - The `egui::Context` to apply the fonts to.
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

/// The main entry point of the application.
///
/// This asynchronous function initializes the eframe native options, sets up shared state
/// for the GUI (output, shell core), and runs the eframe application.
/// It also spawns a background task to handle shutdown signals.
///
/// # Returns
///
/// A `eframe::Result<()>` indicating the success or failure of the application.
#[tokio::main]
async fn main() -> eframe::Result<()> {
    // Configure eframe native options, such as the window size.
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    // Create a oneshot channel for sending a shutdown signal to background tasks.
    let (tx, rx) = oneshot::channel();

    // Run the eframe application.
    let app_result = eframe::run_native(
        "my_cli_tool GUI", // The title of the application window.
        options,
        Box::new(move |cc| {
            // Set up custom fonts for the GUI context.
            setup_fonts(&cc.egui_ctx);

            // Spawn a background task to listen for the shutdown signal.
            // This task will exit when the `rx` receiver receives a message.
            task::spawn(async move {
                let mut rx = rx; // Take ownership of the receiver.
                loop {
                    tokio::select! {
                        _ = &mut rx => {
                            // Received shutdown signal, break the loop.
                            break;
                        }
                        _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                            // Periodically wake up to check for shutdown, but do no other work.
                        }
                    }
                }
            });

            // Create and return the main GUI application instance.
            Ok(Box::new(gui::GuiApp::new(tx)))
        }),
    );

    app_result
}
