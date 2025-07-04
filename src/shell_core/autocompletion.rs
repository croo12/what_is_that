//! This module provides functionality for command autocompletion and suggestions.

use std::path::PathBuf;
use crate::command_history::CommandHistory;
use tokio::fs;

/// Generates command suggestions based on the current input and context.
#[derive(Clone)]
pub struct Autocompleter {
    command_history: CommandHistory,
}

impl Autocompleter {
    pub fn new(command_history: CommandHistory) -> Self {
        Self { command_history }
    }

    /// Provides suggestions based on the current input.
    /// This will include built-in commands, history, and file paths.
    pub async fn get_suggestions(&self, input: &str, current_dir: &PathBuf) -> Vec<String> {
        let mut suggestions = Vec::new();

        let parts: Vec<&str> = input.split_whitespace().collect();
        let command_name = parts.first().unwrap_or(&"");
        let current_arg = parts.last().unwrap_or(&"");

        // If no command is entered yet, suggest built-in commands and history
        if command_name.is_empty() || parts.len() == 1 && !input.ends_with(" ") {
            // 1. Built-in commands
            let built_in_commands = vec!["ls", "cd", "ping", "clear", "open"];
            for cmd in built_in_commands {
                if cmd.starts_with(input) {
                    suggestions.push(cmd.to_string());
                }
            }

            // 2. Command history (if input is empty or matches history)
            if input.is_empty() {
                // If input is empty, suggest recent history
                for cmd in self.command_history.history.iter().rev().take(5) {
                    suggestions.push(cmd.clone());
                }
            } else {
                // If input is not empty, suggest history matching the input
                for cmd in self.command_history.history.iter().rev() {
                    if cmd.starts_with(input) {
                        suggestions.push(cmd.clone());
                    }
                }
            }
        }

        // 3. File system paths for relevant commands
        if (command_name == &"cd" || command_name == &"open" || command_name == &"ls") && !current_arg.is_empty() {
            let mut path_to_read = current_dir.clone();
            let mut file_prefix = String::new();

            if current_arg.contains("/") || current_arg.contains("\\") {
                let path_part = PathBuf::from(current_arg);
                if let Some(parent) = path_part.parent() {
                    path_to_read = current_dir.join(parent);
                    file_prefix = path_part.file_name().unwrap_or_default().to_string_lossy().to_string();
                }
            } else {
                file_prefix = current_arg.to_string();
            }

            if let Ok(mut entries) = fs::read_dir(&path_to_read).await {
                while let Some(entry) = entries.next_entry().await.unwrap() {
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    if file_name.starts_with(&file_prefix) {
                        let full_path = if current_arg.contains("/") || current_arg.contains("\\") {
                            let mut p = PathBuf::from(current_arg);
                            p.pop();
                            p.push(&file_name);
                            p.to_string_lossy().to_string()
                        } else {
                            file_name
                        };
                        suggestions.push(full_path);
                    }
                }
            }
        }

        suggestions.sort_unstable();
        suggestions.dedup();
        suggestions
    }
}