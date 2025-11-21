use clap::Parser;
use std::net::{IpAddr, SocketAddr};
use tokio::io;
use tokio::net::UdpSocket;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Local port
    local_port: u16,

    /// Remote peer address
    remote_host: IpAddr,

    /// Remote port
    remote_port: u16,
}

async fn connect_socket(args: Args) -> io::Result<UdpSocket> {
    let socket = UdpSocket::bind(SocketAddr::from(([0, 0, 0, 0], args.local_port))).await?; // TODO: listen on IPv6 as well
    socket.connect((args.remote_host, args.remote_port)).await?;
    Ok(socket)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let sock = connect_socket(args).await.unwrap();
    nanovpn::Tunnel::new(sock).start().await.unwrap();

    Ok(())
}
