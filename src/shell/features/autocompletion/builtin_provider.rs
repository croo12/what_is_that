//! Suggests built-in commands.

pub(super) async fn get_builtin_suggestions(input: &str) -> Vec<String> {
    let mut builtin_suggestions = Vec::new();
    let built_in_commands = vec!["ls", "cd", "ping", "clear", "open", "mkdir", "rm", "cp", "mv"];
    let parts = shlex::split(input).unwrap_or_default();

    if parts.len() <= 1 && !input.ends_with(' ') {
        let cmd_part = if parts.is_empty() { "" } else { &parts[0] };
        for cmd in &built_in_commands {
            if cmd.starts_with(cmd_part) {
                builtin_suggestions.push(cmd.to_string());
            }
        }
    }
    builtin_suggestions
}

#[cfg(test)]
mod tests {
    use crate::shell::history::CommandHistory;
    use crate::shell::features::autocompletion::Autocompleter;
    use std::path::PathBuf;

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
        
        // Should not suggest anything if there is a space
        let suggestions_with_space = autocompleter.get_suggestions("ls ", &current_dir).await;
        assert!(!suggestions_with_space.contains(&"ls".to_string()));
    }
}
