//! Built-in command to manage environment variables.

use std::collections::HashMap;

/// Handles the `export` command.
///
/// # Arguments
///
/// * `env_vars` - A mutable reference to the `HashMap` storing the environment variables.
/// * `args` - A slice of strings representing the arguments to the command.
///
/// # Returns
///
/// A `String` containing the output of the command.
pub fn export_builtin(env_vars: &mut HashMap<String, String>, args: &[&str]) -> String {
    if args.is_empty() {
        // No arguments, print all environment variables
        if env_vars.is_empty() {
            return "No environment variables defined.\n".to_string();
        }
        let mut output = String::new();
        for (key, value) in env_vars.iter() {
            output.push_str(&format!("export {}='{}'\n", key, value));
        }
        return output;
    }

    for arg in args {
        if let Some((key, value)) = arg.split_once('=') {
            let clean_value = value.trim_matches('\'').trim_matches('"').to_string();
            env_vars.insert(key.to_string(), clean_value);
        } else {
            // If the key is not in `KEY=value` format, we could either
            // treat it as an error, or export it with an empty value.
            // For now, we'll just ignore it, as `export VAR` without `=`
            // is often used to export a variable from the parent shell,
            // a concept our shell doesn't have yet.
        }
    }

    String::new() // No output on successful setting
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_new_env_var() {
        let mut env_vars = HashMap::new();
        let args = vec!["MY_VAR=my_value"];
        let output = export_builtin(&mut env_vars, &args);
        assert!(output.is_empty());
        assert_eq!(env_vars.get("MY_VAR"), Some(&"my_value".to_string()));
    }

    #[test]
    fn test_set_env_var_with_quotes() {
        let mut env_vars = HashMap::new();
        let args = vec!["MY_VAR='hello world'"];
        export_builtin(&mut env_vars, &args);
        assert_eq!(env_vars.get("MY_VAR"), Some(&"hello world".to_string()));
    }

    #[test]
    fn test_print_all_env_vars() {
        let mut env_vars = HashMap::new();
        env_vars.insert("VAR1".to_string(), "value1".to_string());
        env_vars.insert("VAR2".to_string(), "value2".to_string());
        
        let output = export_builtin(&mut env_vars, &[]);
        assert!(output.contains("export VAR1='value1'\n"));
        assert!(output.contains("export VAR2='value2'\n"));
    }
}
