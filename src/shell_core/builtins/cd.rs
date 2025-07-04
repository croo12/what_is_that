use std::env;
use std::path::PathBuf;

pub async fn cd_builtin(current_dir: &mut PathBuf, args: &[&str]) -> String {
    if args.is_empty() {
        return "Usage: cd <directory>\n".to_string();
    }

    let new_dir = args[0];
    let path = current_dir.join(new_dir);

    if !path.is_dir() {
        return format!("cd: '{}': Not a directory\n", new_dir);
    }

    match env::set_current_dir(&path) {
        Ok(_) => {
            *current_dir = path.canonicalize().unwrap();
            String::new()
        }
        Err(e) => format!("cd: '{}': {}\n", new_dir, e),
    }
}
