//! This module provides the built-in `echo` command.

/// Implements the `echo` command, which prints its arguments to the output.
/// It handles basic escape sequences like `\n` and `\t`.
///
/// # Arguments
///
/// * `args` - A slice of string slices, where each element is an argument to `echo`.
///
/// # Returns
///
/// A `String` containing the concatenated arguments, separated by spaces,
/// with escape sequences interpreted.
pub async fn echo_builtin(args: &[&str]) -> String {
    let raw_str = args.join(" ");
    // A simple interpretation of common escape sequences.
    raw_str.replace("\\n", "\n").replace("\\t", "\t")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_echo_builtin_no_args() {
        let output = echo_builtin(&[]).await;
        assert_eq!(output, "");
    }

    #[tokio::test]
    async fn test_echo_builtin_single_arg() {
        let output = echo_builtin(&["hello"]).await;
        assert_eq!(output, "hello");
    }

    #[tokio::test]
    async fn test_echo_builtin_multiple_args() {
        let output = echo_builtin(&["hello", "world", "from", "rust"]).await;
        assert_eq!(output, "hello world from rust");
    }

    #[tokio::test]
    async fn test_echo_builtin_with_special_chars() {
        let output = echo_builtin(&["$PATH", "&&", "||", ">", "output.txt"]).await;
        assert_eq!(output, "$PATH && || > output.txt");
    }

    #[tokio::test]
    async fn test_echo_builtin_with_newline_escape() {
        let output = echo_builtin(&["hello\\nworld"]).await;
        assert_eq!(output, "hello\nworld");
    }

    #[tokio::test]
    async fn test_echo_builtin_with_tab_escape() {
        let output = echo_builtin(&["hello\\tworld"]).await;
        assert_eq!(output, "hello\tworld");
    }
}
