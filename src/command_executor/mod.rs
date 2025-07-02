use tokio::process::Command;
use tokio::io::{self};
use encoding_rs::Encoding;

pub async fn execute_command(command_str: &str) -> String {
    println!("[DEBUG] Executing command: {}", command_str);

    let (shell, shell_arg) = if cfg!(windows) {
        ("cmd.exe", "/C")
    } else {
        ("sh", "-c")
    };

    let output_result = Command::new(shell)
        .arg(shell_arg)
        .arg(command_str)
        .output()
        .await;

    match output_result {
        Ok(output) => {
            let decoder = if cfg!(windows) {
                Encoding::for_label(b"windows-949").unwrap()
            } else {
                Encoding::for_label(b"utf-8").unwrap()
            };
            let (decoded_stdout, _, _) = decoder.decode(&output.stdout);
            let (decoded_stderr, _, _) = decoder.decode(&output.stderr);

            if !output.status.success() {
                format!(
                    "Command failed with exit code: {}\nStdout:\n{}\nStderr:\n{}",
                    output.status.code().unwrap_or(-1),
                    decoded_stdout,
                    decoded_stderr
                )
            } else {
                format!("{}{}", decoded_stdout, decoded_stderr)
            }
        }
        Err(e) => {
            format!("Error executing command: {}\n", e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::execute_command;
    use tokio::io;

    #[tokio::test]
    async fn test_execute_command_echo() -> io::Result<()> {
        let command = if cfg!(windows) {
            "echo Hello from OS!"
        } else {
            "echo Hello from OS!"
        };
        let output = execute_command(command).await;
        println!("Test Output: {}", output);
        assert!(output.contains("Hello from OS!"));
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_command_invalid() -> io::Result<()> {
        let command = "nonexistent_command_12345";
        let output = execute_command(command).await;
        println!("Test Output: {}", output);
        assert!(output.contains("Error executing command:") || output.contains("not found") || output.contains("command not found") || output.contains("실행할 수 있는 프로그램"));
        Ok(())
    }
}
