//! This module provides the core shell functionality, including command execution,
//! directory management, and built-in commands like `ls`, `ping`, and `cd`.

use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use crate::shell::features::git::GitInfo;

pub mod builtins;
pub mod command_executor;
pub mod external;

/// `ShellCore` manages the shell's state, including the current working directory
/// and provides methods for executing commands.
pub struct ShellCore {
    pub current_dir: PathBuf,
    pub git_info: Option<GitInfo>,
    pub aliases: HashMap<String, String>,
    pub env_vars: HashMap<String, String>,
}

impl ShellCore {
    /// Creates a new `ShellCore` instance, initializing the current directory
    /// to the current working directory of the process.
    pub fn new() -> Self {
        let mut core = Self {
            current_dir: dunce::canonicalize(env::current_dir().unwrap()).unwrap(),
            git_info: None,
            aliases: HashMap::new(),
            env_vars: HashMap::new(),
        };
        core.update_git_info();
        core
    }

    /// Updates the Git information based on the current directory.
    pub fn update_git_info(&mut self) {
        self.git_info = crate::shell::features::git::get_git_info(&self.current_dir);
    }

    /// Returns the current working directory of the shell.
    ///
    /// # Returns
    ///
    /// A `PathBuf` representing the current directory.
    pub fn get_current_dir(&self) -> PathBuf {
        self.current_dir.clone()
    }

    /// Executes a given shell command.
    ///
    /// This function parses the command string, identifies built-in commands
    /// (`ls`, `ping`, `cd`), and executes them. If the command is not built-in,
    /// it attempts to execute it as an external system command.
    ///
    /// # Arguments
    ///
    /// * `command_str` - A string slice representing the command to execute.
    ///
    /// # Returns
    ///
    /// A `String` containing the output of the executed command.
    pub async fn execute_shell_command(&mut self, command_str: &str) -> String {
        let result = command_executor::execute_shell_command(self, command_str).await;
        // After a command, especially `cd`, the git info might have changed.
        self.update_git_info();
        result
    }
}

#[cfg(test)]
mod tests {
    use super::{ShellCore};
    use tokio::io;
    use std::path::PathBuf;
    use std::env;

    #[tokio::test]
    async fn test_ls_builtin_current_dir() -> io::Result<()> {
        let mut shell_core = ShellCore::new();
        shell_core.current_dir = dunce::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR"))).unwrap();
        let output = super::builtins::ls::ls_builtin(&shell_core.current_dir, &[]).await;
        assert!(output.contains("Cargo.toml"));
        assert!(output.contains("src"));
        assert!(output.contains("lib"));
        assert!(output.contains("README.md"));
        Ok(())
    }

    #[tokio::test]
    async fn test_ls_builtin_nonexistent_dir() -> io::Result<()> {
        let shell_core = ShellCore::new();
        let output = super::builtins::ls::ls_builtin(&shell_core.current_dir, &["nonexistent_dir_123"]).await;
        println!("Test Output: {}", output);
        assert!(output.contains("No such file or directory"));
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_shell_command_ls() -> io::Result<()> {
        let mut shell_core = ShellCore::new();
        shell_core.current_dir = dunce::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR"))).unwrap();
        println!("DEBUG: shell_core.current_dir = {:?}", shell_core.current_dir);
        let output = shell_core.execute_shell_command("ls").await;
        println!("Test Output: {}", output);
        assert!(output.contains("Cargo.toml"));
        assert!(output.contains("src"));
        assert!(output.contains("lib"));
        assert!(output.contains("README.md"));
        Ok(())
    }

    /*
    #[tokio::test]
    async fn test_execute_shell_command_echo() -> io::Result<()> {
        let mut shell_core = ShellCore::new();
        let command = if cfg!(windows) {
            "echo Hello from OS!"
        } else {
            "echo Hello from OS!"
        };
        let output = shell_core.execute_shell_command(command).await;
        println!("Test Output: {}", output);
        assert!(output.contains("Hello from OS!"));
        Ok(())
    }
    */

    #[tokio::test]
    async fn test_execute_shell_command_invalid() -> io::Result<()> {
        let mut shell_core = ShellCore::new();
        let command = "nonexistent_command_12345";
        let output = shell_core.execute_shell_command(command).await;
        // The new error format is "Error: <command>: command not found"
        assert!(output.contains("Error: nonexistent_command_12345: command not found"));
        Ok(())
    }

    #[tokio::test]
    async fn test_cd_builtin() -> io::Result<()> {
        std::env::set_current_dir(env!("CARGO_MANIFEST_DIR")).unwrap();
        let mut shell_core = ShellCore::new();
        let initial_dir = dunce::canonicalize(&shell_core.current_dir).unwrap();

        // Test cd to a valid directory
        shell_core.execute_shell_command("cd src").await;
        assert_eq!(shell_core.current_dir, dunce::canonicalize(initial_dir.join("src")).unwrap());
        assert_eq!(shell_core.get_current_dir(), dunce::canonicalize(initial_dir.join("src")).unwrap());

        // Test cd back to the parent directory
        shell_core.execute_shell_command("cd ..").await;
        assert_eq!(shell_core.current_dir, initial_dir);
        assert_eq!(shell_core.get_current_dir(), initial_dir);

        // Test cd to a non-existent directory
        let output = shell_core.execute_shell_command("cd nonexistent_dir_123").await;
        assert!(output.contains("cd: 'nonexistent_dir_123': No such file or directory"));
        assert_eq!(shell_core.current_dir, initial_dir);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_current_dir() -> io::Result<()> {
        let initial_dir = dunce::canonicalize(std::env::current_dir().unwrap()).unwrap();
        let shell_core = ShellCore::new();
        assert_eq!(shell_core.get_current_dir(), initial_dir);
        Ok(())
    }

    // This test is ignored because it requires administrator privileges to create raw sockets.
    #[tokio::test]
    #[ignore]
    async fn test_ping_builtin() -> io::Result<()> {
        let _shell_core = ShellCore::new();
        let output = super::builtins::ping::ping_builtin(&["google.com"]).await;
        println!("Test Output: {}", output);
        assert!(output.contains("Reply from"));
        Ok(())
    }
}