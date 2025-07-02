use tokio::process::Command;
use tokio::io;
use encoding_rs::Encoding;
use std::path::Path;

use pnet::packet::icmp::{echo_request, IcmpTypes, IcmpPacket};
use pnet::packet::icmp::checksum;
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::transport::{transport_channel, TransportChannelType, icmp_packet_iter};
use pnet::packet::Packet; // Import the Packet trait
use std::net::ToSocketAddrs;
use std::time::{Duration, Instant};

pub async fn execute_shell_command(command_str: &str) -> String {
    println!("[DEBUG] Executing shell command: {}", command_str);

    let parts: Vec<&str> = command_str.split_whitespace().collect();
    let command_name = parts.first().unwrap_or(&"");
    let args = &parts[1..];

    match *command_name {
        "ls" => ls_builtin(args).await,
        "ping" => ping_builtin(args).await,
        _ => execute_external_command(command_str).await,
    }
}

async fn ls_builtin(args: &[&str]) -> String {
    let path_str = args.first().unwrap_or(&".");
    let path = Path::new(path_str);

    if !path.exists() {
        return format!("ls: cannot access '{}': No such file or directory\n", path_str);
    }

    if !path.is_dir() {
        return format!("{}\n", path_str);
    }

    let mut output = String::new();
    match tokio::fs::read_dir(path).await {
        Ok(mut entries) => {
            while let Some(entry) = entries.next_entry().await.unwrap() {
                output.push_str(&format!("{}\n", entry.file_name().to_string_lossy()));
            }
        }
        Err(e) => {
            output.push_str(&format!("ls: error reading directory '{}': {}\n", path_str, e));
        }
    }
    output
}

async fn ping_builtin(args: &[&str]) -> String {
    if args.is_empty() {
        return "Usage: ping <host>\n".to_string();
    }

    let host = args[0];
    let ip_addr = match host.to_socket_addrs() {
        Ok(mut addrs) => match addrs.next() {
            Some(addr) => addr.ip(),
            None => return format!("ping: unknown host {}\n", host),
        },
        Err(e) => return format!("ping: failed to resolve host {}: {}\n", host, e),
    };

    let (mut tx, mut rx) = match transport_channel(
        4096, // Buffer size
        TransportChannelType::Layer3(IpNextHeaderProtocols::Icmp),
    ) {
        Ok((tx, rx)) => (tx, rx),
        Err(e) => return format!("ping: failed to create transport channel: {}\n", e),
    };

    let mut echo_packet = echo_request::MutableEchoRequestPacket::owned(vec![0; 16]).unwrap();
    echo_packet.set_identifier(1);
    echo_packet.set_sequence_number(1);
    echo_packet.set_icmp_type(IcmpTypes::EchoRequest.into()); // Corrected IcmpTypes::Echo
    
    // Create an IcmpPacket from the MutableEchoRequestPacket for checksum calculation
    let icmp_packet = IcmpPacket::new(echo_packet.packet()).unwrap();
    echo_packet.set_checksum(checksum(&icmp_packet)); // Use imported checksum

    let start_time = Instant::now();

    match tx.send_to(echo_packet.to_immutable(), ip_addr) {
        Ok(_) => {},
        Err(e) => return format!("ping: failed to send packet: {}\n", e),
    }

    // Use tokio::task::spawn_blocking to run the blocking iter.next() call
    let received_addr = match tokio::time::timeout(Duration::from_secs(4), tokio::task::spawn_blocking(move || {
        let mut iter = icmp_packet_iter(&mut rx);
        loop {
            match iter.next() { // Correctly handle Result<...>
                Ok((packet, addr)) => {
                    if addr == ip_addr && packet.get_icmp_type() == IcmpTypes::EchoReply {
                        return Some(addr); // Return only the address
                    }
                },
                Err(e) => {
                    // Handle error from iter.next()
                    eprintln!("ping: error in iter.next(): {}", e);
                    return None;
                },
            }
            // Small sleep to prevent busy-waiting in the blocking thread
            std::thread::sleep(Duration::from_millis(10));
        }
    })).await {
        Ok(Ok(Some(addr))) => addr,
        Ok(Ok(None)) => return "ping: Request timed out (no packet received).\n".to_string(),
        Ok(Err(e)) => return format!("ping: error in blocking task: {}\n", e),
        Err(_) => return "ping: Request timed out (blocking task).\n".to_string(),
    };

    let duration = start_time.elapsed();
    format!("Reply from {}: time={:?}\n", received_addr, duration)
}

async fn execute_external_command(command_str: &str) -> String {
    println!("[DEBUG] Executing external command: {}", command_str);

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
    use super::{execute_shell_command, ls_builtin};
    use tokio::io;
    use std::fs;

    #[tokio::test]
    async fn test_ls_builtin_current_dir() -> io::Result<()> {
        let output = ls_builtin(&[]).await;
        println!("Test Output: {}", output);
        assert!(output.contains("Cargo.toml"));
        assert!(output.contains("src"));
        Ok(())
    }

    #[tokio::test]
    async fn test_ls_builtin_nonexistent_dir() -> io::Result<()> {
        let output = ls_builtin(&["nonexistent_dir_123"]).await;
        println!("Test Output: {}", output);
        assert!(output.contains("No such file or directory"));
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_shell_command_ls() -> io::Result<()> {
        let output = execute_shell_command("ls").await;
        println!("Test Output: {}", output);
        assert!(output.contains("Cargo.toml"));
        assert!(output.contains("src"));
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_shell_command_echo() -> io::Result<()> {
        let command = if cfg!(windows) {
            "echo Hello from OS!"
        } else {
            "echo Hello from OS!"
        };
        let output = execute_shell_command(command).await;
        println!("Test Output: {}", output);
        assert!(output.contains("Hello from OS!"));
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_shell_command_invalid() -> io::Result<()> {
        let command = "nonexistent_command_12345";
        let output = execute_shell_command(command).await;
        println!("Test Output: {}", output);
        assert!(output.contains("Error executing command:") || output.contains("not found") || output.contains("command not found") || output.contains("실행할 수 있는 프로그램"));
        Ok(())
    }

    // #[tokio::test]
    // async fn test_ping_builtin() -> io::Result<()> {
    //     let output = execute_shell_command("ping google.com").await;
    //     println!("Test Output: {}", output);
    //     assert!(output.contains("Reply from"));
    //     Ok(())
    // }
}