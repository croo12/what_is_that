//! This module provides a built-in `grep` command.

use anyhow::{anyhow, Result};
use std::io::{BufRead, BufReader, Read};

/// The core logic for grep, reading from a BufRead source.
fn grep_logic(pattern: &str, mut reader: impl BufRead) -> Result<String> {
    let mut output = String::new();
    let mut line = String::new();

    while reader.read_line(&mut line)? > 0 {
        if line.contains(pattern) {
            output.push_str(&line);
        }
        line.clear();
    }
    Ok(output)
}

/// A simple `grep` implementation that reads from a given input stream.
/// This function is designed to be used in pipelines.
pub async fn grep_builtin(args: &[&str], input: Box<dyn Read + Send>) -> Result<String> {
    if args.is_empty() {
        return Err(anyhow!("grep: missing pattern"));
    }
    let pattern = args[0];

    let reader = BufReader::new(input);
    grep_logic(pattern, reader)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[tokio::test]
    async fn test_grep_builtin_with_matches() {
        let pattern = "hello";
        let input_str = "hello world\ngoodbye world\nhello again\n";
        let input = Box::new(Cursor::new(input_str));
        
        let result = grep_builtin(&[pattern], input).await.unwrap();
        assert_eq!(result, "hello world\nhello again\n");
    }

    #[tokio::test]
    async fn test_grep_builtin_no_matches() {
        let pattern = "rust";
        let input_str = "hello world\ngoodbye world\nhello again";
        let input = Box::new(Cursor::new(input_str));

        let result = grep_builtin(&[pattern], input).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_grep_builtin_no_pattern() {
        let input_str = "hello world";
        let input = Box::new(Cursor::new(input_str));

        let result = grep_builtin(&[], input).await;
        assert!(result.is_err());
    }
}
