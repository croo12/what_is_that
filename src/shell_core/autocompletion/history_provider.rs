//! Suggests commands from history.

use crate::command_history::CommandHistory;

pub(super) async fn get_history_suggestions(
    command_history: &CommandHistory,
    input: &str,
) -> Vec<String> {
    let mut history_suggestions = Vec::new();
    if input.is_empty() {
        for cmd in command_history.history.iter().rev().take(5) {
            history_suggestions.push(cmd.clone());
        }
    } else {
        for cmd in command_history.history.iter().rev() {
            if cmd.starts_with(input) && cmd != input {
                history_suggestions.push(cmd.clone());
            }
        }
    }
    history_suggestions
}

#[cfg(test)]
mod tests {
    use crate::command_history::CommandHistory;
    use crate::shell_core::autocompletion::Autocompleter;
    use std::path::PathBuf;

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
        
        // Should not suggest the exact match
        let suggestions_exact = autocompleter.get_suggestions("cmd1", &current_dir).await;
        assert!(!suggestions_exact.contains(&"cmd1".to_string()));
    }
}
