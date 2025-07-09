//! This module provides the core logic for executing shell commands.

use std::path::PathBuf;
use crate::shell_core::builtins;
use crate::shell_core::external;

pub async fn execute_shell_command(current_dir: &mut PathBuf, command_str: &str) -> String {
    let parts = match shlex::split(command_str) {
        Some(parts) => parts,
        None => return "Error: Invalid command".to_string(),
    };

    if parts.is_empty() {
        return String::new();
    }

    let command_name = &parts[0];
    let args: Vec<&str> = parts.iter().skip(1).map(AsRef::as_ref).collect();

    match command_name.as_str() {
        "ls" => builtins::ls::ls_builtin(current_dir, &args).await,
        "ping" => builtins::ping::ping_builtin(&args).await,
        "cd" => builtins::cd::cd_builtin(current_dir, &args).await,
        "open" => builtins::open::open_builtin(current_dir, &args).await,
        "mkdir" => builtins::mkdir::mkdir_builtin(current_dir, &args).await,
        "rm" => builtins::rm::rm_builtin(current_dir, &args).await,
        "cp" => builtins::cp::cp_builtin(current_dir, &args).await,
        "mv" => builtins::mv::mv_builtin(current_dir, &args).await,
        _ => external::execute_external_command(command_name, &args).await,
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

    /*
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
    */

    #[tokio::test]
    async fn test_execute_unknown_command() -> io::Result<()> {
        let mut current_dir = env::current_dir().unwrap();
        let output = execute_shell_command(&mut current_dir, "nonexistent_command_xyz").await;
        assert!(output.contains("command not found"));
        Ok(())
    }

    // Note: test_ping_builtin and test_open_builtin are not included here
    // as they require specific environment setups (admin privileges for ping, GUI interaction for open)
    // and are already tested in their respective modules or will be tested via integration tests.
}