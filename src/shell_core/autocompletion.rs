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

        let parts = shlex::split(input).unwrap_or_default();
        let command_name = parts.first().map_or("", |s| s.as_str());
        let last_arg = parts.last().map_or("", |s| s.as_str());

        // Get built-in command suggestions
        suggestions.extend(self.get_builtin_suggestions(input, &parts).await);

        // Get command history suggestions
        suggestions.extend(self.get_history_suggestions(input).await);

        // Get file system path suggestions
        suggestions.extend(self.get_filesystem_suggestions(input, current_dir, command_name, last_arg).await);

        suggestions.sort_unstable();
        suggestions.dedup();
        suggestions
    }

    async fn get_builtin_suggestions(&self, input: &str, parts: &[String]) -> Vec<String> {
        let mut builtin_suggestions = Vec::new();
        let built_in_commands = vec!["ls", "cd", "ping", "clear", "open", "mkdir", "rm", "cp", "mv"];
        if parts.len() <= 1 && !input.ends_with(" ") { // Only suggest built-ins if typing the command itself and not ending with space
            for cmd in &built_in_commands {
                if cmd.starts_with(input) {
                    builtin_suggestions.push(cmd.to_string());
                }
            }
        }
        builtin_suggestions
    }

    async fn get_history_suggestions(&self, input: &str) -> Vec<String> {
        let mut history_suggestions = Vec::new();
        if input.is_empty() {
            for cmd in self.command_history.history.iter().rev().take(5) {
                history_suggestions.push(cmd.clone());
            }
        } else {
            for cmd in self.command_history.history.iter().rev() {
                if cmd.starts_with(input) {
                    history_suggestions.push(cmd.clone());
                }
            }
        }
        history_suggestions
    }

    async fn get_filesystem_suggestions(&self, input: &str, current_dir: &PathBuf, _command_name: &str, _last_arg: &str) -> Vec<String> {
        let mut fs_suggestions = Vec::new();
        let mut parts = shlex::split(input).unwrap_or_default();

        if input.is_empty() {
            parts.push("".to_string());
        } else if input.ends_with(' ') {
            parts.push("".to_string());
        }

        let last_part = parts.last().cloned().unwrap_or_default();
        let base_parts = if !parts.is_empty() {
            parts[..parts.len() - 1].to_vec()
        } else {
            vec![]
        };

        let path = PathBuf::from(&last_part);
        
        let (scan_dir, prefix) = if last_part.ends_with('/') || last_part.ends_with('\\') {
            (current_dir.join(&path), "".to_string())
        } else if let Some(parent) = path.parent() {
            (current_dir.join(parent), path.file_name().unwrap_or_default().to_string_lossy().to_string())
        } else {
            (current_dir.clone(), last_part.to_string())
        };

        if let Ok(mut entries) = fs::read_dir(scan_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Some(file_name_os) = entry.path().file_name() {
                    let file_name = file_name_os.to_string_lossy();
                    if file_name.starts_with(&prefix) {
                        let is_dir = entry.file_type().await.map_or(false, |ft| ft.is_dir());
                        
                        let new_last_part = if last_part.ends_with('/') || last_part.ends_with('\\') {
                            format!("{}{}", last_part, file_name)
                        } else {
                            file_name.into_owned()
                        };

                        let mut final_suggestion = if is_dir && !new_last_part.ends_with('/') {
                            format!("{}/", new_last_part)
                        } else {
                            new_last_part
                        };

                        let mut new_parts = base_parts.clone();
                        new_parts.push(final_suggestion);

                        if let Ok(joined) = shlex::try_join(new_parts.iter().map(|s| s.as_str())) {
                            fs_suggestions.push(joined);
                        }
                    }
                }
            }
        }

        fs_suggestions
    }
}

#[cfg(test)]
mod tests {
    use super::{Autocompleter};
    use crate::command_history::CommandHistory;
    use std::path::PathBuf;
    use tokio::fs;
    use std::env;

    #[tokio::test]
    async fn test_builtin_command_suggestions() {
        let history = CommandHistory::new();
        let autocompleter = Autocompleter::new(history);
        let current_dir = PathBuf::from(".");

        let suggestions = autocompleter.get_suggestions("l", &current_dir).await;
        assert!(suggestions.contains(&"ls".to_string()));
        assert!(!suggestions.contains(&"cd".to_string()));

        let suggestions = autocompleter.get_suggestions("o", &current_dir).await;
        assert!(suggestions.contains(&"open".to_string()));
    }

    #[tokio::test]
    async fn test_history_suggestions() {
        let mut history = CommandHistory::new();
        history.add("cmd1".to_string());
        history.add("cmd2".to_string());
        let autocompleter = Autocompleter::new(history);
        let current_dir = PathBuf::from(".");

        let suggestions = autocompleter.get_suggestions("", &current_dir).await;
        assert!(suggestions.contains(&"cmd1".to_string()));
        assert!(suggestions.contains(&"cmd2".to_string()));

        let suggestions = autocompleter.get_suggestions("cmd", &current_dir).await;
        assert!(suggestions.contains(&"cmd1".to_string()));
        assert!(suggestions.contains(&"cmd2".to_string()));
    }

    #[tokio::test]
    async fn test_file_system_suggestions() {
        let history = CommandHistory::new();
        let autocompleter = Autocompleter::new(history);
        let temp_dir = env::temp_dir().join("test_fs_suggestions");
        fs::create_dir_all(&temp_dir).await.unwrap();

        // Create dummy files/directories for testing
        fs::create_dir_all(temp_dir.join("test_dir")).await.unwrap();
        fs::write(temp_dir.join("test_file.txt"), "").await.unwrap();

        let suggestions = autocompleter.get_suggestions("cd test", &temp_dir).await;
        assert!(suggestions.contains(&"cd test_dir/".to_string()));
        assert!(suggestions.contains(&"cd test_file.txt".to_string()));

        // Clean up dummy files/directories
        fs::remove_dir_all(&temp_dir).await.unwrap();
    }

    #[tokio::test]
    async fn test_path_with_slash_suggestions() {
        let history = CommandHistory::new();
        let autocompleter = Autocompleter::new(history);
        let temp_dir = env::temp_dir().join("test_path_with_slash_suggestions");
        fs::create_dir_all(&temp_dir).await.unwrap();

        fs::create_dir_all(temp_dir.join("parent_dir/child_dir")).await.unwrap();
        fs::write(temp_dir.join("parent_dir/file.txt"), "").await.unwrap();

        let suggestions = autocompleter.get_suggestions("cd parent_dir/", &temp_dir).await;
        assert!(suggestions.contains(&"cd parent_dir/child_dir/".to_string()));
        assert!(suggestions.contains(&"cd parent_dir/file.txt".to_string()));

        fs::remove_dir_all(&temp_dir).await.unwrap();
    }

    #[tokio::test]
    async fn test_autocompletion_with_quotes() {
        let history = CommandHistory::new();
        let autocompleter = Autocompleter::new(history);
        let temp_dir = env::temp_dir().join("test_autocompletion_with_quotes");
        fs::create_dir_all(&temp_dir).await.unwrap();

        fs::create_dir_all(temp_dir.join("my folder")).await.unwrap();

        let suggestions = autocompleter.get_suggestions("ls \"my f\"", &temp_dir).await;
        assert!(suggestions.contains(&"ls 'my folder/'".to_string()));

        fs::remove_dir_all(&temp_dir).await.unwrap();
    }
}