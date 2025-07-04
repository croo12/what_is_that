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

        // 3. File system paths
        if let Ok(mut entries) = fs::read_dir(current_dir).await {
            while let Some(entry) = entries.next_entry().await.unwrap() {
                let file_name = entry.file_name().to_string_lossy().to_string();
                if file_name.starts_with(input) {
                    suggestions.push(file_name);
                }
            }
        }

        suggestions.sort_unstable();
        suggestions.dedup();
        suggestions
    }
}
