use std::path::PathBuf;

pub async fn cd_builtin(current_dir: &mut PathBuf, args: &[&str]) -> String {
    if args.len() != 1 {
        return "Usage: cd <directory>\n".to_string();
    }


    let new_dir = args[0];
    let path = current_dir.join(new_dir);

    if !path.exists() {
        return format!("cd: '{}': No such file or directory\n", new_dir);
    }

    if !path.is_dir() {
        return format!("cd: '{}': Not a directory\n", new_dir);
    }

    *current_dir = path.canonicalize().unwrap();
    String::new()
}
