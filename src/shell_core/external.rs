//! This module handles the execution of external commands.

use std::process::{Command, Stdio};
use std::path::PathBuf;
use std::env;

/// Executes an external command that is not a built-in shell command.
/// It searches for the command in the system's PATH.
pub async fn execute_external_command(command: &str, args: &[&str]) -> String {
    match find_executable_in_path(command) {
        Some(path) => {
            match Command::new(&path)
                .args(args)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
            {
                Ok(output) => {
                    if output.status.success() {
                        decode_output(&output.stdout)
                    } else {
                        decode_output(&output.stderr)
                    }
                }
                Err(e) => format!("Error executing command: {}", e),
            }
        }
        None => format!("command not found: {}", command),
    }
}

/// Decodes command output from raw bytes to a String.
/// It attempts to decode using UTF-8, and falls back to a system encoding for Windows.
fn decode_output(bytes: &[u8]) -> String {
    if let Ok(s) = String::from_utf8(bytes.to_vec()) {
        return s;
    }

    let encoding = if cfg!(windows) {
        encoding_rs::EUC_KR
    } else {
        encoding_rs::UTF_8
    };
    
    let (decoded, _, _) = encoding.decode(bytes);
    decoded.into_owned()
}

/// Searches for an executable in the directories listed in the PATH environment variable.
fn find_executable_in_path(command: &str) -> Option<PathBuf> {
    env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths).find_map(|dir| {
            // Append extensions for Windows executables
            let extensions = if cfg!(windows) { vec!["", ".exe", ".cmd", ".bat", ".com"] } else { vec![""] };
            for ext in extensions {
                let full_path = dir.join(format!("{}{}", command, ext));
                if full_path.is_file() {
                    return Some(full_path);
                }
            }
            None
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /*
    // This test is commented out because `echo` is a shell built-in on Windows,
    // not an external executable. The external command executor should not be
    // responsible for handling it. A dedicated built-in for `echo` should be created later.
    #[tokio::test]
    async fn test_execute_echo_command() {
        let command = if cfg!(windows) { "echo" } else { "echo" };
        let output = execute_external_command(command, &["hello", "world"]).await;
        assert!(output.trim().contains("hello world"));
    }
    */

    #[tokio::test]
    async fn test_command_not_found() {
        let output = execute_external_command("nonexistentcommand12345", &[]).await;
        assert!(output.contains("command not found"));
    }
}
