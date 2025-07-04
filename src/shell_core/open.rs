//! Implements the built-in `open` command for opening files and directories.

use std::process::Command;
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

    let mut command = if cfg!(target_os = "windows") {
        let mut cmd = Command::new("cmd");
        cmd.arg("/C").arg("start").arg("").arg(path);
        cmd
    } else if cfg!(target_os = "macos") {
        let mut cmd = Command::new("open");
        cmd.arg(path);
        cmd
    } else if cfg!(target_os = "linux") {
        let mut cmd = Command::new("xdg-open");
        cmd.arg(path);
        cmd
    } else {
        return "open: Not supported on this operating system.\n".to_string();
    };

    match command.spawn() {
        Ok(_) => String::new(),
        Err(e) => format!("open: Failed to open '{}': {}\n", target, e),
    }
}
