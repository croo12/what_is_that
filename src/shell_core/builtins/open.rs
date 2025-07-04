//! Implements the built-in `open` command for opening files and directories.

use std::path::PathBuf;

pub async fn open_builtin(current_dir: &PathBuf, args: &[&str]) -> String {
    if args.is_empty() {
        return "Usage: open <file_or_directory>\n".to_string();
    }

    let target = args[0];
    let path = current_dir.join(target);

    if !path.exists() {
        return format!("open: '{}': No such file or directory\n", target);
    }

    match open::that(&path) {
        Ok(_) => String::new(),
        Err(e) => format!("open: Failed to open '{}': {}\n", target, e),
    }
}