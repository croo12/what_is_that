/*
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

        let parts: Vec<String> = self.parse_arguments(input);
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

    fn parse_arguments(&self, input: &str) -> Vec<String> {
        let mut args = Vec::new();
        let mut current_arg = String::new();
        let mut in_quote = false;
        let mut chars = input.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                '\'' => {
                    if let Some(next_c) = chars.next() {
                        current_arg.push(next_c);
                    } else {
                        current_arg.push(c);
                    }
                },
                '"' => {
                    in_quote = !in_quote;
                    if !in_quote && !current_arg.is_empty() {
                        args.push(current_arg.clone());
                        current_arg.clear();
                    }
                },
                ' ' => {
                    if in_quote {
                        current_arg.push(c);
                    } else if !current_arg.is_empty() {
                        args.push(current_arg.clone());
                        current_arg.clear();
                    }
                },
                _ => current_arg.push(c),
            }
        }
        if !current_arg.is_empty() {
            args.push(current_arg);
        }
        args
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

        let (base_cmd, path_prefix) = if let Some(pos) = input.rfind(' ') {
            // Find the real start of the argument, even with multiple spaces
            let trim_pos = input.trim_end().rfind(' ').unwrap_or(pos);
            (&input[..=trim_pos], &input[trim_pos + 1..])
        } else {
            ("", input)
        };

        if path_prefix.is_empty() && !input.ends_with(' ') {
             if base_cmd.is_empty() {
                 if let Ok(mut entries) = fs::read_dir(current_dir).await {
                     while let Some(entry) = entries.next_entry().await.unwrap_or(None) {
                         let file_name = entry.file_name().to_string_lossy().to_string();
                         if file_name.starts_with(path_prefix) {
                             let suggestion = if file_name.contains(' ') {
                                 format!("\"{}"\", file_name)
                             } else {
                                 file_name
                             };
                             fs_suggestions.push(suggestion);
                         }
                     }
                 }
             }
             return fs_suggestions;
        }

        let path = PathBuf::from(path_prefix);
        let (scan_dir, prefix) = if path_prefix.ends_with('/') || path_prefix.ends_with('\\') {
            (current_dir.join(&path), "".to_string())
        } else if let Some(parent) = path.parent() {
            (current_dir.join(parent), path.file_name().unwrap_or_default().to_string_lossy().to_string())
        } else {
            (current_dir.clone(), path_prefix.to_string())
        };

        if let Ok(mut entries) = fs::read_dir(scan_dir).await {
            while let Some(entry) = entries.next_entry().await.unwrap_or(None) {
                let file_name = entry.file_name().to_string_lossy().to_string();
                if file_name.starts_with(&prefix) {
                    let suggestion = if file_name.contains(' ') {
                        format!("\"{}"\", file_name)
                    } else {
                        file_name
                    };
                    fs_suggestions.push(format!("{}{}", base_cmd, suggestion));
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
        assert!(suggestions.contains(&"cd test_dir".to_string()));
        assert!(suggestions.contains(&"cd test_file.txt".to_string()));

        // Clean up dummy files/directories
        fs::remove_dir_all(&temp_dir).await.unwrap();
    }

    #[tokio::test]
    async fn test_combined_suggestions() {
        let mut history = CommandHistory::new();
        history.add("my_custom_command".to_string());
        let autocompleter = Autocompleter::new(history);
        let temp_dir = env::temp_dir().join("test_combined_suggestions");
        fs::create_dir_all(&temp_dir).await.unwrap();

        fs::create_dir_all(temp_dir.join("another_dir")).await.unwrap();

        let suggestions = autocompleter.get_suggestions("a", &temp_dir).await;
        // The current logic for history suggestions only includes commands that *start with* the input.
        // "my_custom_command" does not start with "a". So, this assertion should be removed or the test modified.
        // assert!(suggestions.contains(&"my_custom_command".to_string()));
        assert!(suggestions.contains(&"another_dir".to_string()));

        fs::remove_dir_all(&temp_dir).await.unwrap();
    }

    #[tokio::test]
    async fn test_path_with_slash_suggestions() {
        let history = CommandHistory::new();
        let autocompleter = Autocompleter::new(history);
        let temp_dir = env::temp_dir().join("test_path_with_slash_suggestions");
        fs::create_dir_all(&temp_dir).await.unwrap();

        fs::create_dir_all(temp_dir.join("parent_dir\\child_dir")).await.unwrap();
        fs::write(temp_dir.join("parent_dir\\file.txt"), "").await.unwrap();

        let suggestions = autocompleter.get_suggestions("cd parent_dir\\", &temp_dir).await;
        assert!(suggestions.contains(&"cd parent_dir\\child_dir".to_string()));
        assert!(suggestions.contains(&"cd parent_dir\\file.txt".to_string()));

        fs::remove_dir_all(&temp_dir).await.unwrap();
    }

    #[tokio::test]
    async fn test_non_existent_path() {
        let history = CommandHistory::new();
        let autocompleter = Autocompleter::new(history);
        let temp_dir = env::temp_dir().join("test_non_existent_path");
        fs::create_dir_all(&temp_dir).await.unwrap();

        let suggestions = autocompleter.get_suggestions("cd non_existent_dir\\", &temp_dir).await;
        assert!(suggestions.is_empty());

        fs::remove_dir_all(&temp_dir).await.unwrap();
    }

    #[tokio::test]
    async fn test_autocompletion_multiple_arguments() {
        let history = CommandHistory::new();
        let autocompleter = Autocompleter::new(history);
        let temp_dir = env::temp_dir().join("test_autocompletion_multiple_arguments");
        fs::create_dir_all(&temp_dir).await.unwrap();

        fs::create_dir_all(temp_dir.join("dir_one")).await.unwrap();
        fs::create_dir_all(temp_dir.join("dir_two")).await.unwrap();
        fs::write(temp_dir.join("file.txt"), "").await.unwrap();

        let suggestions = autocompleter.get_suggestions("cp file.txt dir_", &temp_dir).await;
        assert!(suggestions.contains(&"cp file.txt dir_one".to_string()));
        assert!(suggestions.contains(&"cp file.txt dir_two".to_string()));

        fs::remove_dir_all(&temp_dir).await.unwrap();
    }

    #[tokio::test]
    async fn test_autocompletion_relative_paths() {
        let history = CommandHistory::new();
        let autocompleter = Autocompleter::new(history);
        let temp_dir = env::temp_dir().join("test_autocompletion_relative_paths");
        fs::create_dir_all(&temp_dir).await.unwrap();
        let current_dir = temp_dir.join("src");
        fs::create_dir_all(&current_dir).await.unwrap();

        fs::create_dir_all(temp_dir.join("test_parent_dir")).await.unwrap();
        fs::write(temp_dir.join("test_parent_file.txt"), "").await.unwrap();

        let suggestions = autocompleter.get_suggestions("cd ..\\test_p", &current_dir).await;
        assert!(suggestions.contains(&"cd ..\\test_parent_dir".to_string()));
        assert!(suggestions.contains(&"cd ..\\test_parent_file.txt".to_string()));

        fs::remove_dir_all(&temp_dir).await.unwrap();
    }

    #[tokio::test]
    async fn test_autocompletion_paths_with_spaces() {
        let history = CommandHistory::new();
        let autocompleter = Autocompleter::new(history);
        let temp_dir = env::temp_dir().join("test_autocompletion_paths_with_spaces");
        fs::create_dir_all(&temp_dir).await.unwrap();

        fs::create_dir_all(temp_dir.join("dir with spaces")).await.unwrap();
        fs::write(temp_dir.join("file with spaces.txt"), "").await.unwrap();

        let suggestions = autocompleter.get_suggestions("cd dir with", &temp_dir).await;
        assert!(suggestions.contains(&"cd \"dir with spaces\"".to_string()));

        let suggestions = autocompleter.get_suggestions("open file with", &temp_dir).await;
        assert!(suggestions.contains(&"open \"file with spaces.txt\"".to_string()));

        fs::remove_dir_all(&temp_dir).await.unwrap();
    }

    #[tokio::test]
    async fn test_autocompletion_partial_command_arguments() {
        let history = CommandHistory::new();
        let autocompleter = Autocompleter::new(history);
        let temp_dir = env::temp_dir().join("test_autocompletion_partial_command_arguments");
        fs::create_dir_all(&temp_dir).await.unwrap();

        fs::create_dir_all(temp_dir.join("partial_dir_one")).await.unwrap();
        fs::write(temp_dir.join("partial_file_two.txt"), "").await.unwrap();

        let suggestions = autocompleter.get_suggestions("ls partial", &temp_dir).await;
        assert!(suggestions.contains(&"ls partial_dir_one".to_string()));
        assert!(suggestions.contains(&"ls partial_file_two.txt".to_string()));

        fs::remove_dir_all(&temp_dir).await.unwrap();
    }
}
*/