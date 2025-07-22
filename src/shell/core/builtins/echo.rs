use std::collections::HashMap;
use regex::Regex;

/// Implements the `echo` command, which prints its arguments to the output.
/// It handles basic escape sequences like `\n` and `\t` and expands environment variables.
///
/// # Arguments
///
/// * `args` - A slice of string slices, where each element is an argument to `echo`.
/// * `env_vars` - A reference to the `HashMap` storing the environment variables.
///
/// # Returns
///
/// A `String` containing the concatenated arguments, separated by spaces,
/// with escape sequences and environment variables interpreted.
pub async fn echo_builtin(args: &[&str], env_vars: &HashMap<String, String>) -> String {
    let raw_str = args.join(" ");
    let mut processed_str = raw_str.replace("\\n", "\n").replace("\\t", "\t");

    // Regex to find %VAR% patterns
    let re = Regex::new(r"%([A-Za-z_][A-Za-z0-9_]*?)%").unwrap();

    // Replace environment variables
    processed_str = re.replace_all(&processed_str, |caps: &regex::Captures| {
        let var_name = &caps[1];
        env_vars.get(var_name).map_or(caps[0].to_string(), |val| val.clone())
    }).to_string();

    processed_str
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_echo_builtin_no_args() {
        let env_vars = HashMap::new();
        let output = echo_builtin(&[], &env_vars).await;
        assert_eq!(output, "");
    }

    #[tokio::test]
    async fn test_echo_builtin_single_arg() {
        let env_vars = HashMap::new();
        let output = echo_builtin(&["hello"], &env_vars).await;
        assert_eq!(output, "hello");
    }

    #[tokio::test]
    async fn test_echo_builtin_multiple_args() {
        let env_vars = HashMap::new();
        let output = echo_builtin(&["hello", "world", "from", "rust"], &env_vars).await;
        assert_eq!(output, "hello world from rust");
    }

    #[tokio::test]
    async fn test_echo_builtin_with_special_chars() {
        let env_vars = HashMap::new();
        let output = echo_builtin(&["$PATH", "&&", "||", ">", "output.txt"], &env_vars).await;
        assert_eq!(output, "$PATH && || > output.txt");
    }

    #[tokio::test]
    async fn test_echo_builtin_with_newline_escape() {
        let env_vars = HashMap::new();
        let output = echo_builtin(&["hello\\nworld"], &env_vars).await;
        assert_eq!(output, "hello\nworld");
    }

    #[tokio::test]
    async fn test_echo_builtin_with_tab_escape() {
        let env_vars = HashMap::new();
        let output = echo_builtin(&["hello\\tworld"], &env_vars).await;
        assert_eq!(output, "hello\tworld");
    }

    #[tokio::test]
    async fn test_echo_builtin_env_var_expansion() {
        let mut env_vars = HashMap::new();
        env_vars.insert("MY_VAR".to_string(), "test_value".to_string());
        env_vars.insert("ANOTHER_VAR".to_string(), "another_value".to_string());

        let output = echo_builtin(&["Hello", "%MY_VAR%", "and", "%ANOTHER_VAR%"], &env_vars).await;
        assert_eq!(output, "Hello test_value and another_value");
    }

    #[tokio::test]
    async fn test_echo_builtin_nonexistent_env_var() {
        let env_vars = HashMap::new();
        let output = echo_builtin(&["Hello", "%NON_EXISTENT_VAR%"], &env_vars).await;
        assert_eq!(output, "Hello %NON_EXISTENT_VAR%");
    }

    #[tokio::test]
    async fn test_echo_builtin_env_var_with_no_value() {
        let mut env_vars = HashMap::new();
        env_vars.insert("EMPTY_VAR".to_string(), String::new());
        let output = echo_builtin(&["Value is: %EMPTY_VAR%"], &env_vars).await;
        assert_eq!(output, "Value is: ");
    }
}
