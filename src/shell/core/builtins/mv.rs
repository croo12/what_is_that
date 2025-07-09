use std::path::PathBuf;
use tokio::fs;

pub async fn mv_builtin(current_dir: &PathBuf, args: &[&str]) -> String {
    if args.len() < 2 {
        return "mv: missing file operand\nTry 'mv --help' for more information.\n".to_string();
    }

    let source_path_str = args[0];
    let destination_path_str = args[1];

    let source_path = current_dir.join(source_path_str);
    let destination_path = current_dir.join(destination_path_str);

    if !source_path.exists() {
        return format!("mv: cannot stat '{}': No such file or directory\n", source_path.display());
    }

    match fs::rename(&source_path, &destination_path).await {
        Ok(_) => String::new(),
        Err(e) => format!("mv: cannot move '{}' to '{}': {}\n", source_path.display(), destination_path.display(), e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tokio::fs;
    

    #[tokio::test]
    async fn test_mv_builtin_file() {
        let temp_dir = env::temp_dir().join("test_mv_builtin_file");
        fs::create_dir_all(&temp_dir).await.unwrap();
        let src_file = temp_dir.join("source.txt");
        let dest_file = temp_dir.join("destination.txt");
        fs::write(&src_file, "hello world").await.unwrap();

        let args = ["source.txt", "destination.txt"];
        let output = mv_builtin(&temp_dir, &args).await;

        assert!(output.is_empty(), "Expected no output for successful mv, but got: {}", output);
        assert!(!src_file.exists(), "Source file should not exist");
        assert!(fs::metadata(&dest_file).await.is_ok(), "Destination file should exist");
        assert_eq!(fs::read_to_string(&dest_file).await.unwrap(), "hello world");

        fs::remove_dir_all(&temp_dir).await.unwrap();
    }

    #[tokio::test]
    async fn test_mv_builtin_nonexistent_source() {
        let temp_dir = env::temp_dir().join("test_mv_builtin_nonexistent_source");
        fs::create_dir_all(&temp_dir).await.unwrap();

        let args = ["nonexistent.txt", "destination.txt"];
        let output = mv_builtin(&temp_dir, &args).await;

        assert!(output.contains("No such file or directory"));

        fs::remove_dir_all(&temp_dir).await.unwrap();
    }

    #[tokio::test]
    async fn test_mv_builtin_missing_operand() {
        let temp_dir = env::temp_dir().join("test_mv_builtin_missing_operand");
        fs::create_dir_all(&temp_dir).await.unwrap();

        let args: [&str; 0] = [];
        let output = mv_builtin(&temp_dir, &args).await;

        assert!(output.contains("missing file operand"));

        fs::remove_dir_all(&temp_dir).await.unwrap();
    }
}
