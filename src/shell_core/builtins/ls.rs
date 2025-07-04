use std::path::PathBuf;
use tokio::fs;

pub async fn ls_builtin(current_dir: &PathBuf, args: &[&str]) -> String {
    let path_str = args.first().unwrap_or(&".");
    let path = current_dir.join(path_str);

    if !path.exists() {
        return format!("ls: cannot access '{}': No such file or directory\n", path.display());
    }

    if !path.is_dir() {
        return format!("{}\n", path.display());
    }

    let mut output = String::new();
    match fs::read_dir(path.clone()).await {
        Ok(mut entries) => {
            while let Some(entry) = entries.next_entry().await.unwrap() {
                output.push_str(&format!("{}\n", entry.file_name().to_string_lossy()));
            }
        }
        Err(e) => {
            output.push_str(&format!("ls: error reading directory '{}': {}\n", path.display(), e));
        }
    }
    output
}