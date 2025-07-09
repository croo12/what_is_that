//! This module provides functionality for command autocompletion and suggestions.

mod builtin_provider;
mod history_provider;
mod path_provider;

use crate::shell::history::CommandHistory;
use std::path::PathBuf;

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

        // Get suggestions from all providers concurrently.
        let (builtin_res, history_res, path_res) = tokio::join!(
            builtin_provider::get_builtin_suggestions(input),
            history_provider::get_history_suggestions(&self.command_history, input),
            path_provider::get_filesystem_suggestions(input, current_dir)
        );

        suggestions.extend(builtin_res);
        suggestions.extend(history_res);
        suggestions.extend(path_res);

        suggestions.sort_unstable();
        suggestions.dedup();
        suggestions
    }
}
