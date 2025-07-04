//! This module provides functionality for command autocompletion and suggestions.

use std::path::PathBuf;

use crate::command_history::CommandHistory;

/// Generates command suggestions based on the current input and context.
pub struct Autocompleter {
    command_history: CommandHistory,
}

impl Autocompleter {
    pub fn new(command_history: CommandHistory) -> Self {
        Self { command_history }
    }

    /// Provides suggestions based on the current input.
    /// This will include built-in commands, and potentially history and file paths.
    pub fn get_suggestions(&self, input: &str, _current_dir: &PathBuf) -> Vec<String> {
        let mut suggestions = Vec::new();

        // 1. Built-in commands
        let built_in_commands = vec!["ls", "cd", "ping", "clear"];
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

        // 3. File system paths (for 'cd' and 'ls' like commands)
        // This will be implemented later as it requires async operations and more context.

        suggestions.sort_unstable();
        suggestions.dedup();
        suggestions
    }
}