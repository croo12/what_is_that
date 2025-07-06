use std::path::PathBuf;
use tokio::fs;

pub async fn cp_builtin(current_dir: &PathBuf, args: &[&str]) -> String {
    if args.len() < 2 {
        return "cp: missing file operand\nTry 'cp --help' for more information.\n".to_string();
    }

    let source_path_str = args[0];
    let destination_path_str = args[1];

    let source_path = current_dir.join(source_path_str);
    let destination_path = current_dir.join(destination_path_str);

    if !source_path.exists() {
        return format!("cp: cannot stat '{}': No such file or directory\n", source_path.display());
    }

    if source_path.is_dir() {
        // Recursive copy for directories is not yet implemented
        return format!("cp: -r not specified; omitting directory '{}'\n", source_path.display());
    }

    match fs::copy(&source_path, &destination_path).await {
        Ok(_) => String::new(),
        Err(e) => format!("cp: cannot copy '{}' to '{}': {}\n", source_path.display(), destination_path.display(), e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tokio::fs;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_cp_builtin_file() {
        let temp_dir = env::temp_dir().join("test_cp_builtin_file");
        fs::create_dir_all(&temp_dir).await.unwrap();
        let src_file = temp_dir.join("source.txt");
        let dest_file = temp_dir.join("destination.txt");
        fs::write(&src_file, "hello world").await.unwrap();

        let args = ["source.txt", "destination.txt"];
        let output = cp_builtin(&temp_dir, &args).await;

        assert!(output.is_empty(), "Expected no output for successful cp, but got: {}", output);
        assert!(fs::metadata(&dest_file).await.is_ok(), "Destination file should exist");
        assert_eq!(fs::read_to_string(&dest_file).await.unwrap(), "hello world");

        fs::remove_dir_all(&temp_dir).await.unwrap();
    }

    #[tokio::test]
    async fn test_cp_builtin_nonexistent_source() {
        let temp_dir = env::temp_dir().join("test_cp_builtin_nonexistent_source");
        fs::create_dir_all(&temp_dir).await.unwrap();

        let args = ["nonexistent.txt", "destination.txt"];
        let output = cp_builtin(&temp_dir, &args).await;

        assert!(output.contains("No such file or directory"));

        fs::remove_dir_all(&temp_dir).await.unwrap();
    }

    #[tokio::test]
    async fn test_cp_builtin_directory_source() {
        let temp_dir = env::temp_dir().join("test_cp_builtin_directory_source");
        fs::create_dir_all(&temp_dir).await.unwrap();
        let src_dir = temp_dir.join("source_dir");
        fs::create_dir(&src_dir).await.unwrap();

        let args = ["source_dir", "destination_dir"];
        let output = cp_builtin(&temp_dir, &args).await;

        assert!(output.contains("-r not specified"));

        fs::remove_dir_all(&temp_dir).await.unwrap();
    }

    #[tokio::test]
    async fn test_cp_builtin_missing_operand() {
        let temp_dir = env::temp_dir().join("test_cp_builtin_missing_operand");
        fs::create_dir_all(&temp_dir).await.unwrap();

        let args: [&str; 0] = [];
        let output = cp_builtin(&temp_dir, &args).await;

        assert!(output.contains("missing file operand"));

        fs::remove_dir_all(&temp_dir).await.unwrap();
    }
}
