use std::path::PathBuf;
use tokio::fs;
use chrono::{DateTime, Local};

pub async fn ls_builtin(current_dir: &PathBuf, args: &[&str]) -> String {
    let mut show_hidden = false;
    let mut long_format = false;
    let mut target_path_str = ".";

    for &arg in args {
        if arg.starts_with("-") {
            for char_flag in arg.chars().skip(1) {
                match char_flag {
                    'a' => show_hidden = true,
                    'l' => long_format = true,
                    _ => return format!("ls: invalid option -- '{}'\n", char_flag),
                }
            }
        } else {
            target_path_str = arg;
        }
    }

    let path = current_dir.join(target_path_str);

    if !path.exists() {
        return format!("ls: cannot access '{}': No such file or directory\n", path.display());
    }

    if !path.is_dir() {
        if long_format {
            return format_long_entry(&path).await;
        } else {
            return format!("{}\n", path.display());
        }
    }

    let mut output = String::new();
    let mut entries = match fs::read_dir(path.clone()).await {
        Ok(entries) => entries,
        Err(e) => {
            return format!("ls: error reading directory '{}': {}\n", path.display(), e);
        }
    };

    let mut file_names = Vec::new();
    while let Some(entry) = entries.next_entry().await.unwrap() {
        let file_name = entry.file_name().to_string_lossy().into_owned();
        if !show_hidden && file_name.starts_with(".") {
            continue;
        }
        file_names.push(entry.path());
    }

    file_names.sort();

    for entry_path in file_names {
        if long_format {
            output.push_str(&format_long_entry(&entry_path).await);
        } else {
            output.push_str(&format!("{}\n", entry_path.file_name().unwrap().to_string_lossy()));
        }
    }
    output
}

async fn format_long_entry(path: &PathBuf) -> String {
    let metadata = match fs::metadata(path).await {
        Ok(meta) => meta,
        Err(_) => return String::new(), // Should not happen if path exists
    };

    let file_type = if metadata.is_dir() { "d" } else { "-" };
    let permissions = "rwx------"; // Simplified for now
    let size = metadata.len();
    let modified: DateTime<Local> = DateTime::from(metadata.modified().unwrap());
    let file_name = path.file_name().unwrap().to_string_lossy();

    format!("{}{:<10} {:>8} {} {}\n",
            file_type,
            permissions,
            size,
            modified.format("%b %d %H:%M"),
            file_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tokio::fs;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_ls_builtin_no_args() {
        let temp_dir = env::temp_dir().join("test_ls_no_args");
        fs::create_dir_all(&temp_dir).await.unwrap();
        fs::write(temp_dir.join("file1.txt"), "").await.unwrap();
        fs::create_dir(temp_dir.join("dir1")).await.unwrap();

        let output = ls_builtin(&temp_dir, &[]).await;
        assert!(output.contains("file1.txt"));
        assert!(output.contains("dir1"));
        assert!(!output.contains(".hidden"));

        fs::remove_dir_all(&temp_dir).await.unwrap();
    }

    #[tokio::test]
    async fn test_ls_builtin_a_flag() {
        let temp_dir = env::temp_dir().join("test_ls_a_flag");
        fs::create_dir_all(&temp_dir).await.unwrap();
        fs::write(temp_dir.join("file1.txt"), "").await.unwrap();
        fs::write(temp_dir.join(".hidden"), "").await.unwrap();

        let output = ls_builtin(&temp_dir, &["-a"]).await;
        assert!(output.contains("file1.txt"));
        assert!(output.contains(".hidden"));

        fs::remove_dir_all(&temp_dir).await.unwrap();
    }

    #[tokio::test]
    async fn test_ls_builtin_l_flag() {
        let temp_dir = env::temp_dir().join("test_ls_l_flag");
        fs::create_dir_all(&temp_dir).await.unwrap();
        fs::write(temp_dir.join("file1.txt"), "test content").await.unwrap();

        let output = ls_builtin(&temp_dir, &["-l"]).await;
        assert!(output.contains("file1.txt"));
        assert!(output.contains("rwx------")); // Simplified permissions
        assert!(output.contains("12")); // Size of "test content"
        assert!(output.contains("Jul")); // Month of modification

        fs::remove_dir_all(&temp_dir).await.unwrap();
    }

    #[tokio::test]
    async fn test_ls_builtin_al_flags() {
        let temp_dir = env::temp_dir().join("test_ls_al_flags");
        fs::create_dir_all(&temp_dir).await.unwrap();
        fs::write(temp_dir.join("file1.txt"), "").await.unwrap();
        fs::write(temp_dir.join(".hidden"), "hidden content").await.unwrap();

        let output = ls_builtin(&temp_dir, &["-al"]).await;
        assert!(output.contains("file1.txt"));
        assert!(output.contains(".hidden"));
        assert!(output.contains("rwx------"));
        assert!(output.contains("14")); // Size of "hidden content"

        fs::remove_dir_all(&temp_dir).await.unwrap();
    }

    #[tokio::test]
    async fn test_ls_builtin_invalid_flag() {
        let temp_dir = env::temp_dir().join("test_ls_invalid_flag");
        fs::create_dir_all(&temp_dir).await.unwrap();

        let output = ls_builtin(&temp_dir, &["-x"]).await;
        assert!(output.contains("ls: invalid option -- 'x'"));

        fs::remove_dir_all(&temp_dir).await.unwrap();
    }

    #[tokio::test]
    async fn test_ls_builtin_target_path() {
        let temp_dir = env::temp_dir().join("test_ls_target_path");
        fs::create_dir_all(&temp_dir).await.unwrap();
        let sub_dir = temp_dir.join("sub_dir");
        fs::create_dir(&sub_dir).await.unwrap();
        fs::write(sub_dir.join("sub_file.txt"), "").await.unwrap();

        let output = ls_builtin(&temp_dir, &["sub_dir"]).await;
        assert!(output.contains("sub_file.txt"));
        assert!(!output.contains("sub_dir")); // Should not list itself

        fs::remove_dir_all(&temp_dir).await.unwrap();
    }

    #[tokio::test]
    async fn test_ls_builtin_file_with_l_flag() {
        let temp_dir = env::temp_dir().join("test_ls_file_with_l_flag");
        fs::create_dir_all(&temp_dir).await.unwrap();
        let file_path = temp_dir.join("single_file.txt");
        fs::write(&file_path, "file content").await.unwrap();

        let output = ls_builtin(&temp_dir, &["-l", "single_file.txt"]).await;
        assert!(output.contains("single_file.txt"));
        assert!(output.contains("file content".len().to_string().as_str()));
        assert!(output.starts_with("-")); // Should indicate it's a file

        fs::remove_dir_all(&temp_dir).await.unwrap();
    }
}
