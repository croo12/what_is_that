//! This module provides the core logic for executing shell commands.

use anyhow::{anyhow, Context, Result};
use std::fs::File;
use std::io::{Cursor, Write};
use std::process::Stdio;
use crate::shell::core::builtins;
use crate::shell::core::ShellCore;
use tokio::process::Command as TokioCommand;

// Data structures for parsing
#[derive(Debug, PartialEq, Clone)]
struct Command {
    name: String,
    args: Vec<String>,
}

#[derive(Debug, PartialEq, Clone)]
enum Redirection {
    ToFile(String),
}

#[derive(Debug, PartialEq, Clone)]
struct Pipeline {
    commands: Vec<Command>,
    redirection: Option<Redirection>,
}

// Parser function
fn parse_line(line: &str) -> Result<Pipeline, String> {
    let mut commands = Vec::new();
    let mut redirection = None;

    let line_part = match line.rsplit_once('>') {
        Some((left, right)) => {
            let filename = right.trim();
            if filename.is_empty() { return Err("Redirection filename is missing.".to_string()); }
            redirection = Some(Redirection::ToFile(filename.to_string()));
            left
        }
        None => line,
    };

    for part in line_part.split('|') {
        let trimmed_part = part.trim();
        if trimmed_part.is_empty() { return Err("Empty command in pipeline.".to_string()); }
        let args = shlex::split(trimmed_part).ok_or_else(|| format!("Invalid quoting: '{}'", trimmed_part))?;
        if args.is_empty() { return Err("Empty command in pipeline.".to_string()); }
        commands.push(Command { name: args[0].clone(), args: args.into_iter().skip(1).collect() });
    }

    if commands.is_empty() { return Err("No commands provided.".to_string()); }
    Ok(Pipeline { commands, redirection })
}

// --- New Execution Logic ---

async fn execute_pipeline_async(shell_core: &mut ShellCore, pipeline: Pipeline) -> Result<String> {
    let mut input_data = Vec::new();
    let mut last_command_output: Option<Vec<u8>> = None;

    let Pipeline { commands, redirection } = pipeline;
    let num_commands = commands.len();

    for (i, command) in commands.into_iter().enumerate() {
        let is_last_command = i == num_commands - 1;
        let args: Vec<&str> = command.args.iter().map(AsRef::as_ref).collect();

        let command_result_str = match command.name.as_str() {
            "ls" => Ok(builtins::ls::ls_builtin(&shell_core.current_dir, &args).await),
            "echo" => Ok(builtins::echo::echo_builtin(&args, &shell_core.env_vars).await),
            "ping" => Ok(builtins::ping::ping_builtin(&args).await),
            "grep" => {
                let cursor = Cursor::new(input_data.clone());
                builtins::grep::grep_builtin(&args, Box::new(cursor)).await
            }
            "cat" => builtins::cat::cat_builtin(&shell_core.current_dir, &args).await,
            "alias" => Ok(builtins::alias::alias_builtin(&mut shell_core.aliases, &args)),
            "unalias" => {
                let mut unalias_args = vec!["unalias"];
                unalias_args.extend_from_slice(&args);
                Ok(builtins::alias::alias_builtin(&mut shell_core.aliases, &unalias_args))
            }
            "export" => Ok(builtins::export::export_builtin(&mut shell_core.env_vars, &args)),
            "unset" => Ok(builtins::unset::unset_builtin(&mut shell_core.env_vars, &args)),
            "cd" => Ok(builtins::cd::cd_builtin(&mut shell_core.current_dir, &args).await),
            "open" => Ok(builtins::open::open_builtin(&shell_core.current_dir, &args).await),
            "mkdir" => Ok(builtins::mkdir::mkdir_builtin(&shell_core.current_dir, &args).await),
            "rm" => Ok(builtins::rm::rm_builtin(&shell_core.current_dir, &args).await),
            "cp" => Ok(builtins::cp::cp_builtin(&shell_core.current_dir, &args).await),
            "mv" => Ok(builtins::mv::mv_builtin(&shell_core.current_dir, &args).await),
            _ => {
                // External commands
                let mut cmd = TokioCommand::new(&command.name);
                cmd.args(&command.args)
                   .current_dir(&shell_core.current_dir)
                   .stdin(Stdio::piped())
                   .stdout(Stdio::piped())
                   .stderr(Stdio::piped());

                // Set environment variables for the external command
                for (key, value) in &shell_core.env_vars {
                    cmd.env(key, value);
                }

                let mut child = match cmd.spawn() {
                    Ok(child) => child,
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                        return Err(anyhow!("{}: command not found", command.name));
                    }
                    Err(e) => return Err(e).context(format!("Failed to spawn command '{}'", command.name)),
                };
                
                if let Some(mut stdin) = child.stdin.take() {
                    use tokio::io::AsyncWriteExt;
                    stdin.write_all(&input_data).await?;
                }

                let output = child.wait_with_output().await?;
                if !output.status.success() {
                    return Err(anyhow!(String::from_utf8_lossy(&output.stderr).into_owned()));
                }
                return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
            }
        };

        let output_str = command_result_str?;
        let current_command_output_bytes = output_str.into_bytes();

        if is_last_command {
            if let Some(Redirection::ToFile(ref filename)) = redirection {
                let mut file = File::create(shell_core.current_dir.join(filename))
                    .context("Failed to create redirection file")?;
                file.write_all(&current_command_output_bytes)?;
                last_command_output = Some(Vec::new()); // No output to stdout if redirected
            } else {
                last_command_output = Some(current_command_output_bytes);
            }
        } else {
            input_data = current_command_output_bytes;
        }
    }

    Ok(last_command_output.map_or(String::new(), |bytes| String::from_utf8_lossy(&bytes).into_owned()))
}

pub async fn execute_shell_command(shell_core: &mut ShellCore, command_str: &str) -> String {
    if command_str.trim().is_empty() {
        return String::new();
    }

    // Alias expansion
    let mut parts = shlex::split(command_str).unwrap_or_default();
    if parts.is_empty() {
        return String::new();
    }

    let expanded_command_str = if let Some(expanded) = shell_core.aliases.get(&parts[0]) {
        parts[0] = expanded.clone();
        parts.join(" ")
    } else {
        command_str.to_string()
    };

    let pipeline = match parse_line(&expanded_command_str) {
        Ok(p) => p,
        Err(e) => return e,
    };
    
    match execute_pipeline_async(shell_core, pipeline).await {
        Ok(output) => output,
        Err(e) => format!("Error: {}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell::core::ShellCore;
    
    use std::fs;
    use tokio::io;

    #[tokio::test]
    async fn test_builtin_grep_in_pipeline() -> io::Result<()> {
        let mut shell_core = ShellCore::new();
        let command = "echo \"hello\nworld\nhello rust\" | grep hello";
        let output = execute_shell_command(&mut shell_core, command).await;
        assert_eq!(output.trim(), "hello\nhello rust");
        Ok(())
    }

    #[tokio::test]
    async fn test_pipeline_with_redirection() -> io::Result<()> {
        let mut shell_core = ShellCore::new();
        let test_file = "test_pipe_output.txt";
        let command = "echo \"apple\nbanana\napple pie\" | grep apple";
        let full_command = format!("{} > {}", command, test_file);

        let output = execute_shell_command(&mut shell_core, &full_command).await;
        assert!(output.is_empty(), "Output should be empty, but was: {}", output);

        let file_content = fs::read_to_string(shell_core.current_dir.join(test_file))?;
        assert_eq!(file_content.trim(), "apple\napple pie");

        fs::remove_file(shell_core.current_dir.join(test_file))?;
        Ok(())
    }

    #[tokio::test]
    async fn test_ls_redirection() -> io::Result<()> {
        let mut shell_core = ShellCore::new();
        let test_file = "ls_output.txt";
        
        let output = execute_shell_command(&mut shell_core, &format!("ls > {}", test_file)).await;
        assert!(output.is_empty(), "Output to shell should be empty for redirection");

        let file_content = fs::read_to_string(shell_core.current_dir.join(test_file))?;
        assert!(file_content.contains("Cargo.toml"), "File should contain Cargo.toml");
        assert!(file_content.contains("src"), "File should contain src");

        fs::remove_file(shell_core.current_dir.join(test_file))?;
        Ok(())
    }

    #[tokio::test]
    async fn test_three_stage_pipeline() -> io::Result<()> {
        let mut shell_core = ShellCore::new();
        let command = "echo \"apple\nbanana\napple pie\nblueberry\" | grep apple | grep pie";
        let output = execute_shell_command(&mut shell_core, command).await;
        assert_eq!(output.trim(), "apple pie");
        Ok(())
    }

    #[tokio::test]
    async fn test_pipeline_error_in_middle() -> io::Result<()> {
        let mut shell_core = ShellCore::new();
        let command = "echo 'hello' | nonexistentcommand | grep hello";
        let output = execute_shell_command(&mut shell_core, command).await;
        assert!(output.contains("Error: nonexistentcommand: command not found"));
        Ok(())
    }

    #[tokio::test]
    async fn test_pipeline_with_quoted_args() -> io::Result<()> {
        let mut shell_core = ShellCore::new();
        let command = "echo 'hello \"world\"' | grep 'hello \"world\"'";
        let output = execute_shell_command(&mut shell_core, command).await;
        assert_eq!(output.trim(), "hello \"world\"");
        Ok(())
    }

    #[tokio::test]
    async fn test_env_var_management() -> io::Result<()> {
        let mut shell_core = ShellCore::new();

        // 1. Set an environment variable
        let output = execute_shell_command(&mut shell_core, "export MY_VAR=test_value").await;
        assert!(output.is_empty());

        println!("DEBUG: output: {}", output);

        // 2. Check if it's listed by `export`
        let output = execute_shell_command(&mut shell_core, "export").await;
        println!("DEBUG: Output from 'export': '{}'", output);
        assert!(output.contains("export MY_VAR=test_value"));

        // 3. Check if `echo` expands it correctly
        let output = execute_shell_command(&mut shell_core, "echo %MY_VAR%").await;
        assert_eq!(output.trim(), "test_value");

        // 4. Unset the environment variable
        let output = execute_shell_command(&mut shell_core, "unset MY_VAR").await;
        assert!(output.is_empty());

        // 5. Check if it's no longer listed by `export`
        let output = execute_shell_command(&mut shell_core, "export").await;
        assert!(!output.contains("export MY_VAR=test_value"));

        // 6. Check if `echo` no longer expands it
        let output = execute_shell_command(&mut shell_core, "echo %MY_VAR%").await;
        assert_eq!(output.trim(), "%MY_VAR%");

        Ok(())
    }
}
