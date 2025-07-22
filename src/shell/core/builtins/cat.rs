//! Built-in command to concatenate and display file contents.

use anyhow::{anyhow, Result};
use std::fs;
use std::path::PathBuf;

/// Handles the `cat` command.
///
/// Reads the content of specified files and returns them as a single string.
///
/// # Arguments
///
/// * `current_dir` - The current working directory.
/// * `args` - A slice of strings representing the arguments to the command (file paths).
///
/// # Returns
///
/// A `Result<String>` containing the concatenated file contents on success,
/// or an error message if a file cannot be read.
pub async fn cat_builtin(current_dir: &PathBuf, args: &[&str]) -> Result<String> {
    if args.is_empty() {
        return Err(anyhow!("cat: missing operand"));
    }

    let mut output = String::new();
    for arg in args {
        let path = current_dir.join(arg);
        match fs::read_to_string(&path) {
            Ok(content) => {
                output.push_str(&content);
            }
            Err(e) => {
                return Err(anyhow!("cat: {}: {}", path.display(), e));
            }
        }
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_cat_single_file() -> Result<()> {
        let mut file = NamedTempFile::new()?;
        writeln!(file, "Hello, world!")?;
        let path = file.path().to_path_buf();
        let current_dir = env::current_dir()?;

        let output = cat_builtin(&current_dir, &[path.to_str().unwrap()]).await?;
        assert_eq!(output.trim(), "Hello, world!");
        Ok(())
    }

    #[tokio::test]
    async fn test_cat_multiple_files() -> Result<()> {
        let mut file1 = NamedTempFile::new()?;
        writeln!(file1, "Line 1")?;
        let path1 = file1.path().to_path_buf();

        let mut file2 = NamedTempFile::new()?;
        writeln!(file2, "Line 2")?;
        let path2 = file2.path().to_path_buf();

        let current_dir = env::current_dir()?;

        let output = cat_builtin(&current_dir, &[path1.to_str().unwrap(), path2.to_str().unwrap()]).await?;
        assert_eq!(output.trim(), "Line 1
Line 2");
        Ok(())
    }

    #[tokio::test]
    async fn test_cat_nonexistent_file() -> Result<()> {
        let current_dir = env::current_dir()?;
        let result = cat_builtin(&current_dir, &["nonexistent_file.txt"]).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        // Check for common "file not found" phrases across OSes
        let is_file_not_found_error = err_msg.contains("No such file or directory") ||
                                       err_msg.contains("cannot find the file specified") ||
                                       err_msg.contains("The system cannot find the file specified") ||
                                       err_msg.contains("지정된 파일을 찾을 수 없습니다."); // Windows specific Korean message

        assert!(is_file_not_found_error, "Expected file not found error, but got: {}", err_msg);
        Ok(())
    }

    #[tokio::test]
    async fn test_cat_missing_operand() -> Result<()> {
        let current_dir = env::current_dir()?;
        let result = cat_builtin(&current_dir, &[]).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "cat: missing operand");
        Ok(())
    }
}
