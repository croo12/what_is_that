//! This module provides the core logic for executing shell commands.

use std::path::PathBuf;
use crate::shell_core::builtins;
use crate::shell_core::external;

pub async fn execute_shell_command(current_dir: &mut PathBuf, command_str: &str) -> String {
    println!("[DEBUG] Executing shell command: {}", command_str);

    let parts: Vec<&str> = command_str.split_whitespace().collect();
    let command_name = parts.first().unwrap_or(&"");
    let args = &parts[1..];

    match *command_name {
        "ls" => builtins::ls::ls_builtin(current_dir, args).await,
        "ping" => builtins::ping::ping_builtin(args).await,
        "cd" => builtins::cd::cd_builtin(current_dir, args).await,
        "open" => builtins::open::open_builtin(current_dir, args).await,
        "mkdir" => builtins::mkdir::mkdir_builtin(current_dir, args).await,
        "rm" => builtins::rm::rm_builtin(current_dir, args).await,
        _ => external::execute_external_command(current_dir, command_str).await,
    }
}

#[cfg(test)]
mod tests {
    use super::execute_shell_command;
    use std::env;
    use tokio::io;

    #[tokio::test]
    async fn test_execute_ls_command() -> io::Result<()> {
        let mut current_dir = env::current_dir().unwrap();
        let output = execute_shell_command(&mut current_dir, "ls").await;
        assert!(output.contains("Cargo.toml"));
        assert!(output.contains("src"));
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_cd_command() -> io::Result<()> {
        let mut current_dir = env::current_dir().unwrap();
        let initial_dir = current_dir.clone();

        let output = execute_shell_command(&mut current_dir, "cd src").await;
        assert!(output.is_empty());
        assert_eq!(current_dir, initial_dir.join("src").canonicalize().unwrap());

        let output = execute_shell_command(&mut current_dir, "cd ..").await;
        assert!(output.is_empty());
        assert_eq!(current_dir, initial_dir.canonicalize().unwrap());
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_echo_command() -> io::Result<()> {
        let mut current_dir = env::current_dir().unwrap();
        let command = if cfg!(windows) {
            "echo Hello from external!"
        } else {
            "echo Hello from external!"
        };
        let output = execute_shell_command(&mut current_dir, command).await;
        assert!(output.contains("Hello from external!"));
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_unknown_command() -> io::Result<()> {
        let mut current_dir = env::current_dir().unwrap();
        let output = execute_shell_command(&mut current_dir, "nonexistent_command_xyz").await;
        assert!(output.contains("Error executing command:") || output.contains("not found") || output.contains("command not found") || output.contains("실행할 수 있는 프로그램"));
        Ok(())
    }

    // Note: test_ping_builtin and test_open_builtin are not included here
    // as they require specific environment setups (admin privileges for ping, GUI interaction for open)
    // and are already tested in their respective modules or will be tested via integration tests.
}