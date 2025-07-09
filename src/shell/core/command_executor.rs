//! This module provides the core logic for executing shell commands.

use anyhow::{anyhow, Context, Result};
use std::fs::File;
use std::io::{Cursor, Write};
use std::path::PathBuf;
use std::process::Stdio;
use crate::shell::core::builtins;
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

async fn execute_pipeline_async(current_dir: &mut PathBuf, pipeline: Pipeline) -> Result<String> {
    let mut input_data = Vec::new(); // Start with empty input for the first command
    let mut final_output = String::new();

    // Destructure the pipeline to avoid partial move errors
    let Pipeline { commands, redirection } = pipeline;
    let num_commands = commands.len();

    for (i, command) in commands.into_iter().enumerate() {
        let is_last_command = i == num_commands - 1;
        let args: Vec<&str> = command.args.iter().map(AsRef::as_ref).collect();

        let output_data = match command.name.as_str() {
            "echo" => builtins::echo::echo_builtin(&args).await.into_bytes(),
            "grep" => {
                // Clone input_data to avoid moving it, ensuring ownership consistency across match arms.
                let cursor = Cursor::new(input_data.clone());
                builtins::grep::grep_builtin(&args, Box::new(cursor)).await?.into_bytes()
            }
            "cd" if i == 0 => {
                final_output = builtins::cd::cd_builtin(current_dir, &args).await;
                Vec::new()
            }
            _ => { // External commands
                let mut cmd = TokioCommand::new(&command.name);
                cmd.args(&command.args)
                   .current_dir(&*current_dir)
                   .stdin(Stdio::piped())
                   .stdout(Stdio::piped())
                   .stderr(Stdio::piped());

                let mut child = match cmd.spawn() {
                    Ok(child) => child,
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::NotFound {
                            return Err(anyhow!("{}: command not found", command.name));
                        } else {
                            return Err(e).context(format!("Failed to spawn command '{}'", command.name));
                        }
                    }
                };
                
                if let Some(mut stdin) = child.stdin.take() {
                    use tokio::io::AsyncWriteExt;
                    stdin.write_all(&input_data).await?;
                }

                let output = child.wait_with_output().await?;
                if !output.status.success() {
                    return Err(anyhow!(String::from_utf8_lossy(&output.stderr).into_owned()));
                }
                output.stdout
            }
        };

        if is_last_command {
            if let Some(Redirection::ToFile(ref filename)) = redirection {
                let mut file = File::create(current_dir.join(filename))
                    .context("Failed to create redirection file")?;
                file.write_all(&output_data)?;
                final_output = String::new();
            } else {
                final_output = String::from_utf8_lossy(&output_data).into_owned();
            }
        } else {
            input_data = output_data;
        }
    }

    Ok(final_output)
}

pub async fn execute_shell_command(current_dir: &mut PathBuf, command_str: &str) -> String {
    if command_str.trim().is_empty() {
        return String::new();
    }

    let pipeline = match parse_line(command_str) {
        Ok(p) => p,
        Err(e) => return e,
    };
    
    if pipeline.commands.len() == 1 && pipeline.redirection.is_none() {
        let command = &pipeline.commands[0];
        let args: Vec<&str> = command.args.iter().map(AsRef::as_ref).collect();
        match command.name.as_str() {
            "cd" => return builtins::cd::cd_builtin(current_dir, &args).await,
            "open" => return builtins::open::open_builtin(current_dir, &args).await,
            "ls" => return builtins::ls::ls_builtin(current_dir, &args).await,
            "mkdir" => return builtins::mkdir::mkdir_builtin(current_dir, &args).await,
            "rm" => return builtins::rm::rm_builtin(current_dir, &args).await,
            "cp" => return builtins::cp::cp_builtin(current_dir, &args).await,
            "mv" => return builtins::mv::mv_builtin(current_dir, &args).await,
            "ping" => return builtins::ping::ping_builtin(&args).await,
            _ => {}
        }
    }

    match execute_pipeline_async(current_dir, pipeline).await {
        Ok(output) => output,
        Err(e) => format!("Error: {}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use tokio::io;

    #[tokio::test]
    async fn test_builtin_grep_in_pipeline() -> io::Result<()> {
        let mut current_dir = env::current_dir()?;
        // shlex will treat "hello\nworld\nhello rust" as a single argument for echo.
        // The `echo` builtin will then process the `\n` and `grep` will receive multi-line input.
        let command = "echo \"hello\\nworld\\nhello rust\" | grep hello";
        let output = execute_shell_command(&mut current_dir, command).await;
        assert_eq!(output.trim(), "hello\nhello rust");
        Ok(())
    }

    #[tokio::test]
    async fn test_pipeline_with_redirection() -> io::Result<()> {
        let mut current_dir = env::current_dir()?;
        let test_file = "test_pipe_output.txt";
        let command = "echo \"apple\\nbanana\\napple pie\" | grep apple";
        let full_command = format!("{} > {}", command, test_file);

        let output = execute_shell_command(&mut current_dir, &full_command).await;
        assert!(output.is_empty(), "Output should be empty, but was: {}", output);

        let file_content = fs::read_to_string(current_dir.join(test_file))?;
        assert_eq!(file_content.trim(), "apple\napple pie");

        fs::remove_file(current_dir.join(test_file))?;
        Ok(())
    }
}