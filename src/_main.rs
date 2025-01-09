use std::net::{Ipv4Addr, SocketAddr};
use tokio::net::UdpSocket;
use tokio_tun::Tun;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::str::FromStr;
use std::sync::Arc;

struct VpnConfig {
    interface_name: String,
    remote_host: String,
    remote_port: u16,
    local_vpn_ip: String,
    remote_vpn_ip: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = VpnConfig {
        interface_name: "tun0".to_string(),
        remote_host: "188.166.74.116".to_string(),
        // remote_host: "185.75.238.3".to_string(),
        remote_port: 1194,
        local_vpn_ip: "10.0.0.1".to_string(),
        remote_vpn_ip: "10.0.0.2".to_string(),
    };


    let tun = Tun::builder()
        .name(&config.interface_name)
        .up()
        .address(Ipv4Addr::from_str(&config.local_vpn_ip).unwrap()) //&config.local_vpn_ip
        .destination(Ipv4Addr::from_str(&config.remote_vpn_ip).unwrap())
        .mtu(1500)
        .try_build()?;

    let socket = UdpSocket::bind("0.0.0.0:1194").await?;
    socket.connect((config.remote_host, config.remote_port)).await?;

    let (mut tun_reader, mut tun_writer) = tokio::io::split(tun);


    let socket = Arc::new(socket);

    let socket_clone = socket.clone();

    // Handle outgoing traffic (TUN to UDP)
    let outgoing = tokio::spawn(async move {
        let mut buffer = vec![0u8; 1500];
        loop {
            match tun_reader.read(&mut buffer).await {
                Ok(n) if n > 0 => {
                    if let Err(e) = encapsulate_and_send(&socket_clone, &buffer[..n]).await {
                        eprintln!("Error sending packet: {}", e);
                    }
                }
                Ok(_) => continue,
                Err(e) => {
                    eprintln!("Error reading from TUN: {}", e);
                    break;
                }
            }
        }
    });

    // Handle incoming traffic (UDP to TUN)
    let incoming = tokio::spawn(async move {
        let mut buffer = vec![0u8; 1500];
        loop {
            match socket.recv(&mut buffer).await {
                Ok(n) => {
                    if let Err(e) = decapsulate_and_write(&mut tun_writer, &buffer[..n]).await {
                        eprintln!("Error writing to TUN: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("Error receiving UDP packet: {}", e);
                    break;
                }
            }
        }
    });

    // Wait for both tasks to complete (they run indefinitely unless an error occurs)
    tokio::try_join!(outgoing, incoming)?;

    Ok(())
}

async fn encapsulate_and_send(socket: &UdpSocket, packet: &[u8]) -> std::io::Result<()> {
    // TODO: Implement packet encapsulation
    // For now, just forward the packet as-is
    socket.send(packet).await?;
    Ok(())
}

async fn decapsulate_and_write<W: AsyncWriteExt + Unpin>(tun_writer: &mut W, packet: &[u8]) -> std::io::Result<()> {
    // TODO: Implement packet decapsulation
    // For now, just write the packet as-is to the TUN interface
    tun_writer.write_all(packet).await?;
    Ok(())
}