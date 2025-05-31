use std::net::{IpAddr, SocketAddr, TcpStream};
use ipnet::Ipv4Net;
use std::time::Duration;

pub fn scan_subnet(subnet: &str, port: u16) -> std::io::Result<()> {
    let net: Ipv4Net = subnet.parse().unwrap();

    for host in net.hosts() {
        let socket = SocketAddr::new(IpAddr::V4(host), port);
        if TcpStream::connect_timeout(&socket, Duration::from_millis(40)).is_ok() {
            println!("Connected to {}", socket);
        } else {
            println!("Failed to connect to {}", socket);
        }
    }
    Ok(())
}

