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
        _ => external::execute_external_command(current_dir, command_str).await,
    }
}
