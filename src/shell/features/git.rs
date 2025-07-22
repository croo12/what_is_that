//! This module provides functionality for interacting with Git repositories.

use git2::{Repository, StatusOptions};
use std::path::Path;

/// Represents information about a Git repository.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitInfo {
    pub branch_name: String,
    pub has_changes: bool,
}

/// Attempts to find a Git repository at the given path and, if found,
/// returns information about its current state.
pub fn get_git_info(current_dir: &Path) -> Option<GitInfo> {
    // Discover the repository by searching upwards from the current directory
    let repo = match Repository::discover(current_dir) {
        Ok(repo) => repo,
        Err(_) => return None, // Not a git repository
    };

    // Get the current branch name
    let branch_name = get_current_branch(&repo).unwrap_or_else(|| "HEAD".to_string());

    // Check for any changes in the working directory
    let has_changes = has_uncommitted_changes(&repo);

    Some(GitInfo {
        branch_name,
        has_changes,
    })
}

/// Finds the name of the current branch.
fn get_current_branch(repo: &Repository) -> Option<String> {
    let head = repo.head().ok()?;
    if head.is_branch() {
        head.shorthand().map(String::from)
    } else {
        // Detached HEAD state, return short commit hash
        let commit = head.peel_to_commit().ok()?;
        let id = commit.id();
        Some(format!("{:.7}", id))
    }
}

/// Checks if there are any uncommitted changes (new, modified, deleted, etc.).
fn has_uncommitted_changes(repo: &Repository) -> bool {
    let mut opts = StatusOptions::new();
    opts.include_untracked(true).recurse_untracked_dirs(true);

    if let Ok(statuses) = repo.statuses(Some(&mut opts)) {
        !statuses.is_empty()
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use git2::{Repository, Signature};

    // Helper to create a temporary git repository for testing
    fn create_test_repo(path: &Path) -> Repository {
        let repo = Repository::init(path).unwrap();
        let mut config = repo.config().unwrap();
        config.set_str("user.name", "Test User").unwrap();
        config.set_str("user.email", "test@example.com").unwrap();
        let signature = Signature::now("Test User", "test@example.com").unwrap();

        // Create an initial commit
        let tree_id = {
            let mut index = repo.index().unwrap();
            index.write_tree().unwrap()
        };
        repo.commit(Some("HEAD"), &signature, &signature, "Initial commit", &repo.find_tree(tree_id).unwrap(), &[]).unwrap();
        
        repo
    }

    #[test]
    fn test_get_git_info_in_repo() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo_path = temp_dir.path();
        create_test_repo(repo_path);

        let info = get_git_info(repo_path).unwrap();
        assert_eq!(info.branch_name, "main"); // git init now defaults to 'main'
        assert!(!info.has_changes);
    }

    #[test]
    fn test_no_git_info_outside_repo() {
        let temp_dir = tempfile::tempdir().unwrap();
        assert!(get_git_info(temp_dir.path()).is_none());
    }

    #[test]
    fn test_has_changes_detects_modification() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo_path = temp_dir.path();
        let repo = create_test_repo(repo_path);

        // Create a file and add it to the index
        let file_path = repo_path.join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "hello").unwrap();
        
        let mut index = repo.index().unwrap();
        index.add_path(Path::new("test.txt")).unwrap();
        index.write().unwrap();

        // Now modify it
        writeln!(file, "world").unwrap();

        let info = get_git_info(repo_path).unwrap();
        assert!(info.has_changes);
    }
}
