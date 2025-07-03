//! This module provides functionality for managing command history in a shell-like application.
//! It allows adding commands, navigating through the history (up and down), and resetting the history index.

/// `CommandHistory` stores a list of commands entered by the user
/// and keeps track of the current position when navigating through the history.
pub struct CommandHistory {
    /// A vector storing the history of commands as strings.
    history: Vec<String>,
    /// The current index in the history when navigating. `None` if not navigating.
    current_index: Option<usize>,
}

impl CommandHistory {
    /// Creates a new, empty `CommandHistory` instance.
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            current_index: None,
        }
    }

    /// Adds a new command to the history.
    ///
    /// The command is only added if it's not empty and not a duplicate of the last command.
    /// After adding, the `current_index` is reset to `None`.
    ///
    /// # Arguments
    ///
    /// * `command` - The command string to add to the history.
    pub fn add(&mut self, command: String) {
        if !command.is_empty() && self.history.last() != Some(&command) {
            self.history.push(command);
        }
        self.current_index = None;
    }

    /// Navigates up through the command history.
    ///
    /// # Returns
    ///
    /// An `Option<&str>` containing the command string if navigation is successful,
    /// or `None` if at the beginning of the history or history is empty.
    pub fn navigate_up(&mut self) -> Option<&str> {
        if self.history.is_empty() {
            return None;
        }

        let new_index = match self.current_index {
            Some(index) => {
                if index > 0 {
                    Some(index - 1)
                } else {
                    Some(0)
                }
            }
            None => Some(self.history.len() - 1),
        };
        self.current_index = new_index;
        new_index.map(|i| self.history[i].as_str())
    }

    /// Navigates down through the command history.
    ///
    /// # Returns
    ///
    /// An `Option<&str>` containing the command string if navigation is successful,
    /// or `None` if at the end of the history.
    pub fn navigate_down(&mut self) -> Option<&str> {
        if self.history.is_empty() {
            return None;
        }

        let new_index = match self.current_index {
            Some(index) => {
                if index < self.history.len() - 1 {
                    Some(index + 1)
                } else {
                    None // Reached the end of history, clear input
                }
            }
            None => None, // No history to navigate down from
        };
        self.current_index = new_index;
        new_index.map(|i| self.history[i].as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::CommandHistory;

    #[test]
    fn test_new_command_history() {
        let history = CommandHistory::new();
        assert!(history.history.is_empty());
        assert!(history.current_index.is_none());
    }

    #[test]
    fn test_add_command() {
        let mut history = CommandHistory::new();
        history.add("cmd1".to_string());
        assert_eq!(history.history, vec!["cmd1"]);
        assert!(history.current_index.is_none());

        history.add("cmd2".to_string());
        assert_eq!(history.history, vec!["cmd1", "cmd2"]);

        // Test adding duplicate command
        history.add("cmd2".to_string());
        assert_eq!(history.history, vec!["cmd1", "cmd2"]);

        // Test adding empty command
        history.add("".to_string());
        assert_eq!(history.history, vec!["cmd1", "cmd2"]);
    }

    #[test]
    fn test_navigate_up() {
        let mut history = CommandHistory::new();
        assert!(history.navigate_up().is_none()); // Empty history

        history.add("cmd1".to_string());
        history.add("cmd2".to_string());
        history.add("cmd3".to_string());

        assert_eq!(history.navigate_up(), Some("cmd3"));
        assert_eq!(history.current_index, Some(2));

        assert_eq!(history.navigate_up(), Some("cmd2"));
        assert_eq!(history.current_index, Some(1));

        assert_eq!(history.navigate_up(), Some("cmd1"));
        assert_eq!(history.current_index, Some(0));

        assert_eq!(history.navigate_up(), Some("cmd1")); // At the beginning
        assert_eq!(history.current_index, Some(0));
    }

    #[test]
    fn test_navigate_down() {
        let mut history = CommandHistory::new();
        history.add("cmd1".to_string());
        history.add("cmd2".to_string());
        history.add("cmd3".to_string());

        // Navigate up first to set current_index
        history.navigate_up(); // cmd3
        history.navigate_up(); // cmd2
        history.navigate_up(); // cmd1

        assert_eq!(history.navigate_down(), Some("cmd2"));
        assert_eq!(history.current_index, Some(1));

        assert_eq!(history.navigate_down(), Some("cmd3"));
        assert_eq!(history.current_index, Some(2));

        assert!(history.navigate_down().is_none()); // At the end
        assert!(history.current_index.is_none());

        // Test navigate down from empty history (should return None)
        let mut empty_history = CommandHistory::new();
        assert!(empty_history.navigate_down().is_none());
    }

    #[test]
    fn test_add_after_navigation() {
        let mut history = CommandHistory::new();
        history.add("cmd1".to_string());
        history.add("cmd2".to_string());
        history.navigate_up(); // cmd2
        history.add("cmd3".to_string());
        assert_eq!(history.history, vec!["cmd1", "cmd2", "cmd3"]);
        assert!(history.current_index.is_none());
    }
}
