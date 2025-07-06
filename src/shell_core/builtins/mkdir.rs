use std::path::PathBuf;
use tokio::fs;

pub async fn mkdir_builtin(current_dir: &PathBuf, args: &[&str]) -> String {
    if args.is_empty() {
        return "mkdir: missing operand\n".to_string();
    }

    let mut output = String::new();
    for &path_str in args {
        let path = current_dir.join(path_str);
        if let Err(e) = fs::create_dir(&path).await {
            output.push_str(&format!("mkdir: cannot create directory '{}': {}\n", path.display(), e));
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
    async fn test_mkdir_builtin() {
        let temp_dir = env::temp_dir().join("test_mkdir_builtin");
        fs::create_dir_all(&temp_dir).await.unwrap();

        let new_dir_name = "new_test_dir";
        let args = [new_dir_name];
        
        let output = mkdir_builtin(&temp_dir, &args).await;
        
        assert!(output.is_empty(), "Expected no output for successful mkdir, but got: {}", output);

        let new_dir_path = temp_dir.join(new_dir_name);
        assert!(fs::metadata(&new_dir_path).await.is_ok(), "Directory should have been created");
        assert!(fs::metadata(&new_dir_path).await.unwrap().is_dir(), "Path should be a directory");

        fs::remove_dir_all(&temp_dir).await.unwrap();
    }
}
