use std::env;
use std::path::PathBuf;

pub async fn cd_builtin(current_dir: &mut PathBuf, args: &[&str]) -> String {
    if args.is_empty() {
        return "Usage: cd <directory>\n".to_string();
    }

    let new_dir = args[0];
    let path = current_dir.join(new_dir);

    if path.is_dir() {
        *current_dir = path.canonicalize().unwrap();
        String::new()
    } else {
        format!("cd: '{}': Not a directory\n", new_dir)
    }
}
