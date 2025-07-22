//! Built-in command to set and display environment variables.

use std::collections::HashMap;

/// Handles the `export` command.
///
/// Sets environment variables or displays them.
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
            println!("DEBUG: export_builtin returning 'No environment variables defined in this session.'");
            return "No environment variables defined in this session.\n".to_string();
        }
        let mut output = String::new();
        for (key, value) in env_vars.iter() {
            output.push_str(&format!("export {}={}\n", key, value));
        }
        return output;
    }

    let mut new_vars_set = 0;
    for arg in args {
        if let Some((key, value)) = arg.split_once('=') {
            // Set environment variable
            env_vars.insert(key.to_string(), value.to_string());
            new_vars_set += 1;
        } else {
            // If not in `key=value` format, check if it's a key of an existing var to print
            if let Some(value) = env_vars.get(*arg) {
                return format!("export {}={}\n", arg, value);
            } else {
                return format!("export: {}: not found\n", arg);
            }
        }
    }

    if new_vars_set > 0 {
        String::new() // No output on successful setting
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_set_new_var() {
        let mut env_vars = HashMap::new();
        let args = vec!["MY_VAR=test_value"];
        let output = export_builtin(&mut env_vars, &args);
        assert!(output.is_empty());
        assert_eq!(env_vars.get("MY_VAR"), Some(&"test_value".to_string()));
    }

    #[test]
    fn test_export_display_all_vars() {
        let mut env_vars = HashMap::new();
        env_vars.insert("VAR1".to_string(), "value1".to_string());
        env_vars.insert("VAR2".to_string(), "value2".to_string());
        
        let output = export_builtin(&mut env_vars, &[]);
        assert!(output.contains("export VAR1=value1\n"));
        assert!(output.contains("export VAR2=value2\n"));
    }

    #[test]
    fn test_export_display_single_var() {
        let mut env_vars = HashMap::new();
        env_vars.insert("MY_VAR".to_string(), "test_value".to_string());
        let args = vec!["MY_VAR"];
        let output = export_builtin(&mut env_vars, &args);
        assert_eq!(output, "export MY_VAR=test_value\n");
    }

    #[test]
    fn test_export_var_not_found() {
        let mut env_vars = HashMap::new();
        let args = vec!["NON_EXISTENT_VAR"];
        let output = export_builtin(&mut env_vars, &args);
        assert_eq!(output, "export: NON_EXISTENT_VAR: not found\n");
    }

    #[test]
    fn test_export_overwrite_var() {
        let mut env_vars = HashMap::new();
        env_vars.insert("MY_VAR".to_string(), "old_value".to_string());
        let args = vec!["MY_VAR=new_value"];
        let output = export_builtin(&mut env_vars, &args);
        assert!(output.is_empty());
        assert_eq!(env_vars.get("MY_VAR"), Some(&"new_value".to_string()));
    }
}