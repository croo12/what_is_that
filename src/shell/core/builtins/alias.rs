//! Built-in command to manage aliases.

use std::collections::HashMap;

/// Handles the `alias` and `unalias` commands.
///
/// # Arguments
///
/// * `aliases` - A mutable reference to the `HashMap` storing the aliases.
/// * `args` - A slice of strings representing the arguments to the command.
///
/// # Returns
///
/// A `String` containing the output of the command.
pub fn alias_builtin(aliases: &mut HashMap<String, String>, args: &[&str]) -> String {
    if args.is_empty() {
        // No arguments, print all aliases
        if aliases.is_empty() {
            return "No aliases defined.\n".to_string();
        }
        let mut output = String::new();
        for (alias, command) in aliases.iter() {
            output.push_str(&format!("alias {}='{}'\n", alias, command));
        }
        return output;
    }

    // Handle `unalias`
    if args[0] == "unalias" {
        if args.len() < 2 {
            return "unalias: usage: unalias <alias_name>\n".to_string();
        }
        let alias_name = args[1];
        if aliases.remove(alias_name).is_some() {
            return format!("Alias '{}' removed.\n", alias_name);
        } else {
            return format!("unalias: {}: not found\n", alias_name);
        }
    }

    // Handle `alias name=value`
    let mut new_aliases = 0;
    for arg in args {
        if let Some((name, value)) = arg.split_once('=') {
            if value.is_empty() {
                // Unset alias if value is empty
                aliases.remove(name);
            } else {
                // Set alias, removing quotes if present
                let clean_value = if value.starts_with('(') && value.ends_with('(') {
                    value[1..value.len() - 1].to_string()
                } else if value.starts_with('"') && value.ends_with('"') {
                    value[1..value.len() - 1].to_string()
                } else {
                    value.to_string()
                };
                aliases.insert(name.to_string(), clean_value);
                new_aliases += 1;
            }
        } else {
            // If not in `name=value` format, check if it's a name of an existing alias to print
            if let Some(command) = aliases.get(*arg) {
                return format!("alias {}='{}'\n", arg, command);
            } else {
                return format!("alias: {}: not found\n", arg);
            }
        }
    }

    if new_aliases > 0 {
        String::new() // No output on successful setting
    } else {
        // This case is for when `alias` is called with an argument that is not a `name=value` pair and not an existing alias.
        // The loop above already handles the error message, but we need a default return.
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_new_alias() {
        let mut aliases = HashMap::new();
        let args = vec!["ll=ls -l"];
        let output = alias_builtin(&mut aliases, &args);
        assert!(output.is_empty());
        assert_eq!(aliases.get("ll"), Some(&"ls -l".to_string()));
    }

    #[test]
    fn test_set_alias_with_quotes() {
        let mut aliases = HashMap::new();
        let args = vec!["greet=\"echo 'Hello World'\""];
        alias_builtin(&mut aliases, &args);
        assert_eq!(aliases.get("greet"), Some(&"echo 'Hello World'".to_string()));
    }

    #[test]
    fn test_print_all_aliases() {
        let mut aliases = HashMap::new();
        aliases.insert("ll".to_string(), "ls -l".to_string());
        aliases.insert("c".to_string(), "clear".to_string());
        
        let output = alias_builtin(&mut aliases, &[]);
        assert!(output.contains("alias ll='ls -l'\n"));
        assert!(output.contains("alias c='clear'\n"));
    }

    #[test]
    fn test_print_single_alias() {
        let mut aliases = HashMap::new();
        aliases.insert("ll".to_string(), "ls -l".to_string());
        let args = vec!["ll"];
        let output = alias_builtin(&mut aliases, &args);
        assert_eq!(output, "alias ll='ls -l'\n");
    }

    #[test]
    fn test_alias_not_found() {
        let mut aliases = HashMap::new();
        let args = vec!["nonexistent"];
        let output = alias_builtin(&mut aliases, &args);
        assert_eq!(output, "alias: nonexistent: not found\n");
    }

    #[test]
    fn test_unalias() {
        let mut aliases = HashMap::new();
        aliases.insert("ll".to_string(), "ls -l".to_string());
        
        let args = vec!["unalias", "ll"];
        let output = alias_builtin(&mut aliases, &args);
        assert_eq!(output, "Alias 'll' removed.\n");
        assert!(aliases.get("ll").is_none());
    }

    #[test]
    fn test_unalias_not_found() {
        let mut aliases = HashMap::new();
        let args = vec!["unalias", "nonexistent"];
        let output = alias_builtin(&mut aliases, &args);
        assert_eq!(output, "unalias: nonexistent: not found\n");
    }

    #[test]
    fn test_unalias_usage() {
        let mut aliases = HashMap::new();
        let args = vec!["unalias"];
        let output = alias_builtin(&mut aliases, &args);
        assert_eq!(output, "unalias: usage: unalias <alias_name>\n");
    }
}
