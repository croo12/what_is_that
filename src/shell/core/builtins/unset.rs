//! Built-in command to unset environment variables.

use std::collections::HashMap;

/// Handles the `unset` command.
///
/// Removes environment variables from the current session.
///
/// # Arguments
///
/// * `env_vars` - A mutable reference to the `HashMap` storing the environment variables.
/// * `args` - A slice of strings representing the arguments to the command (variable names).
///
/// # Returns
///
/// A `String` containing the output of the command (usually empty on success,
/// or an error message if a variable is not found).
pub fn unset_builtin(env_vars: &mut HashMap<String, String>, args: &[&str]) -> String {
    if args.is_empty() {
        return "unset: usage: unset <variable_name>\n".to_string();
    }

    let mut removed_count = 0;
    for arg in args {
        if env_vars.remove(*arg).is_some() {
            removed_count += 1;
        } else {
            return format!("unset: {}: not found\n", arg);
        }
    }

    if removed_count > 0 {
        String::new() // No output on successful unsetting
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unset_existing_var() {
        let mut env_vars = HashMap::new();
        env_vars.insert("MY_VAR".to_string(), "test_value".to_string());
        
        let args = vec!["MY_VAR"];
        let output = unset_builtin(&mut env_vars, &args);
        assert!(output.is_empty());
        assert!(env_vars.get("MY_VAR").is_none());
    }

    #[test]
    fn test_unset_nonexistent_var() {
        let mut env_vars = HashMap::new();
        let args = vec!["NON_EXISTENT_VAR"];
        let output = unset_builtin(&mut env_vars, &args);
        assert_eq!(output, "unset: NON_EXISTENT_VAR: not found\n");
    }

    #[test]
    fn test_unset_multiple_vars() {
        let mut env_vars = HashMap::new();
        env_vars.insert("VAR1".to_string(), "value1".to_string());
        env_vars.insert("VAR2".to_string(), "value2".to_string());

        let args = vec!["VAR1", "VAR2"];
        let output = unset_builtin(&mut env_vars, &args);
        assert!(output.is_empty());
        assert!(env_vars.get("VAR1").is_none());
        assert!(env_vars.get("VAR2").is_none());
    }

    #[test]
    fn test_unset_usage() {
        let mut env_vars = HashMap::new();
        let args = vec![];
        let output = unset_builtin(&mut env_vars, &args);
        assert_eq!(output, "unset: usage: unset <variable_name>\n");
    }
}
