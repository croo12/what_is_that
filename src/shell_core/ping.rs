

use std::net::ToSocketAddrs;
use std::time::{Duration, Instant};
use pnet::packet::icmp::{echo_request, echo_reply, IcmpTypes, IcmpPacket};
use pnet::packet::icmp::checksum;
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::transport::{transport_channel, TransportChannelType, TransportProtocol, icmp_packet_iter};
use pnet::packet::Packet;



pub async fn ping_builtin(args: &[&str]) -> String {
    if args.is_empty() {
        return "Usage: ping <host>\n".to_string();
    }

    let host = args[0];
    // Clean up host string to remove http(s):// prefixes and paths
    let cleaned_host = host.trim_start_matches("https://").trim_start_matches("http://").split('/').next().unwrap_or("");
    let ip_addr = match cleaned_host.to_socket_addrs() {
        Ok(mut addrs) => match addrs.next() {
            Some(addr) => addr.ip(),
            None => return format!("ping: unknown host {}\n", host),
        },
        Err(e) => return format!("ping: failed to resolve host {}: {}\n", host, e),
    };

    // Set up transport channel for ICMP
    let protocol = TransportProtocol::Ipv4(IpNextHeaderProtocols::Icmp);
    let (mut tx, mut rx) = match transport_channel(4096, TransportChannelType::Layer4(protocol)) {
        Ok((tx, rx)) => (tx, rx),
        Err(e) => return format!("ping: failed to create transport channel: {}\n", e),
    };

    // Create and send ICMP echo request packet
    let mut echo_packet = echo_request::MutableEchoRequestPacket::owned(vec![0; 16]).unwrap();
    echo_packet.set_identifier(1);
    echo_packet.set_sequence_number(1);
    echo_packet.set_icmp_type(IcmpTypes::EchoRequest.into());
    
    let icmp_packet = IcmpPacket::new(echo_packet.packet()).unwrap();
    let checksum = checksum(&icmp_packet);
    echo_packet.set_checksum(checksum);

    let start_time = Instant::now();

    match tx.send_to(echo_packet.to_immutable(), ip_addr) {
        Ok(_) => {},
        Err(e) => return format!("ping: failed to send packet: {}\n", e),
    }

    // Wait for ICMP echo reply
    let received_addr = match tokio::time::timeout(Duration::from_secs(4), tokio::task::spawn_blocking(move || {
        let mut iter = icmp_packet_iter(&mut rx);
        loop {
            match iter.next() {
                Ok((packet, addr)) => {
                    // Verify the reply matches the sent request
                    if let Some(echo_reply) = echo_reply::EchoReplyPacket::new(packet.packet()) {
                        if addr == ip_addr && echo_reply.get_identifier() == 1 && echo_reply.get_sequence_number() == 1 {
                            return Some(addr);
                        }
                    }
                },
                Err(e) => {
                    eprintln!("ping: error in iter.next(): {}", e);
                    return None;
                },
            }
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
