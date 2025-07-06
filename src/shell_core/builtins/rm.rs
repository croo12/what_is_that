use std::path::PathBuf;
use tokio::fs;

pub async fn rm_builtin(current_dir: &PathBuf, args: &[&str]) -> String {
    if args.is_empty() {
        return "rm: missing operand\n".to_string();
    }

    let mut output = String::new();
    for &path_str in args {
        let path = current_dir.join(path_str);
        if !path.exists() {
            output.push_str(&format!("rm: cannot remove '{}': No such file or directory\n", path.display()));
            continue;
        }

        if path.is_dir() {
            if let Err(e) = fs::remove_dir_all(&path).await {
                output.push_str(&format!("rm: cannot remove directory '{}': {}\n", path.display(), e));
            }
        } else {
            if let Err(e) = fs::remove_file(&path).await {
                output.push_str(&format!("rm: cannot remove file '{}': {}\n", path.display(), e));
            }
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tokio::fs;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_rm_builtin_file() {
        let temp_dir = env::temp_dir().join("test_rm_builtin_file");
        fs::create_dir_all(&temp_dir).await.unwrap();
        let file_path = temp_dir.join("test_file.txt");
        fs::write(&file_path, "test content").await.unwrap();

        let args = ["test_file.txt"];
        let output = rm_builtin(&temp_dir, &args).await;

        assert!(output.is_empty(), "Expected no output for successful rm, but got: {}", output);
        assert!(!file_path.exists(), "File should have been removed");

        fs::remove_dir_all(&temp_dir).await.unwrap();
    }

    #[tokio::test]
    async fn test_rm_builtin_dir() {
        let temp_dir = env::temp_dir().join("test_rm_builtin_dir");
        fs::create_dir_all(&temp_dir).await.unwrap();
        let dir_path = temp_dir.join("test_dir");
        fs::create_dir(&dir_path).await.unwrap();

        let args = ["test_dir"];
        let output = rm_builtin(&temp_dir, &args).await;

        assert!(output.is_empty(), "Expected no output for successful rm, but got: {}", output);
        assert!(!dir_path.exists(), "Directory should have been removed");

        fs::remove_dir_all(&temp_dir).await.unwrap();
    }
}