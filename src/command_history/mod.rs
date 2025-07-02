pub struct CommandHistory {
    history: Vec<String>,
    current_index: Option<usize>,
}

impl CommandHistory {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            current_index: None,
        }
    }

    pub fn add(&mut self, command: String) {
        if !command.is_empty() && self.history.last() != Some(&command) {
            self.history.push(command);
        }
        self.current_index = None;
    }

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

    pub fn reset_index(&mut self) {
        self.current_index = None;
    }
}
