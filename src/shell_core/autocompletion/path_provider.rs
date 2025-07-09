//! Suggests file system paths.

use std::path::{Path, PathBuf};
use tokio::fs;

pub(super) async fn get_filesystem_suggestions(
    input: &str,
    current_dir: &Path,
) -> Vec<String> {
    let mut fs_suggestions = Vec::new();
    let mut parts = shlex::split(input).unwrap_or_default();

    if input.is_empty() {
        parts.push("".to_string());
    } else if input.ends_with(' ') {
        parts.push("".to_string());
    }

    let last_part = parts.last().cloned().unwrap_or_default();
    let base_parts = if !parts.is_empty() {
        parts[..parts.len() - 1].to_vec()
    } else {
        vec![]
    };

    let path = PathBuf::from(&last_part);
    
    let (scan_dir, prefix) = if last_part.ends_with('/') || last_part.ends_with('\\') {
        (current_dir.join(&path), "".to_string())
    } else if let Some(parent) = path.parent() {
        (current_dir.join(parent), path.file_name().unwrap_or_default().to_string_lossy().to_string())
    } else {
        (current_dir.to_path_buf(), last_part.to_string())
    };

    if let Ok(mut entries) = fs::read_dir(scan_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Some(file_name_os) = entry.path().file_name() {
                let file_name = file_name_os.to_string_lossy();
                if file_name.starts_with(&prefix) {
                    let is_dir = entry.file_type().await.map_or(false, |ft| ft.is_dir());
                    
                    let new_last_part = if last_part.ends_with('/') || last_part.ends_with('\\') {
                        format!("{}{}", last_part, file_name)
                    } else if let Some(parent) = path.parent() {
                        parent.join(file_name.as_ref()).to_string_lossy().into_owned()
                    } else {
                        file_name.into_owned()
                    };

                    let final_suggestion_part = if is_dir && !new_last_part.ends_with('/') {
                        format!("{}/", new_last_part)
                    } else {
                        new_last_part
                    };

                    let mut new_parts = base_parts.clone();
                    new_parts.push(final_suggestion_part);

                    if let Ok(joined) = shlex::try_join(new_parts.iter().map(|s| s.as_str())) {
                        fs_suggestions.push(joined);
                    }
                }
            }
        }
    }

    fs_suggestions
}

#[cfg(test)]
mod tests {
    use crate::command_history::CommandHistory;
    use crate::shell_core::autocompletion::Autocompleter;
    use std::env;
    use tokio::fs;
    use std::path::PathBuf;

    async fn setup_test_dir(test_name: &str) -> PathBuf {
        let temp_dir = env::temp_dir().join("autocompletion_tests").join(test_name);
        let _ = fs::remove_dir_all(&temp_dir).await;
        fs::create_dir_all(&temp_dir).await.unwrap();
        temp_dir
    }
    
    #[tokio::test]
    async fn test_file_system_suggestions() {
        let history = CommandHistory::new();
        let autocompleter = Autocompleter::new(history);
        let temp_dir = setup_test_dir("test_fs_suggestions").await;

        fs::create_dir_all(temp_dir.join("test_dir")).await.unwrap();
        fs::write(temp_dir.join("test_file.txt"), "").await.unwrap();

        let suggestions = autocompleter.get_suggestions("cd test", &temp_dir).await;
        assert!(suggestions.contains(&"cd test_dir/".to_string()));
        assert!(suggestions.contains(&"cd test_file.txt".to_string()));

        fs::remove_dir_all(&temp_dir).await.unwrap();
    }

    #[tokio::test]
    async fn test_path_with_slash_suggestions() {
        let history = CommandHistory::new();
        let autocompleter = Autocompleter::new(history);
        let temp_dir = setup_test_dir("test_path_with_slash_suggestions").await;

        fs::create_dir_all(temp_dir.join("parent_dir/child_dir")).await.unwrap();
        fs::write(temp_dir.join("parent_dir/file.txt"), "").await.unwrap();

        let suggestions = autocompleter.get_suggestions("cd parent_dir/", &temp_dir).await;
        assert!(suggestions.contains(&"cd parent_dir/child_dir/".to_string()));
        assert!(suggestions.contains(&"cd parent_dir/file.txt".to_string()));

        fs::remove_dir_all(&temp_dir).await.unwrap();
    }

    #[tokio::test]
    async fn test_autocompletion_with_quotes() {
        let history = CommandHistory::new();
        let autocompleter = Autocompleter::new(history);
        let temp_dir = setup_test_dir("test_autocompletion_with_quotes").await;

        fs::create_dir_all(temp_dir.join("my folder")).await.unwrap();

        let suggestions = autocompleter.get_suggestions("ls \"my f\"", &temp_dir).await;
        assert!(suggestions.contains(&"ls 'my folder/'".to_string()));

        fs::remove_dir_all(&temp_dir).await.unwrap();
    }
}
