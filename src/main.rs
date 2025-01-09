use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use clap::{Parser};
use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::UdpSocket;
use tokio_tun::Tun;

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

fn init_tun() -> Tun {
    Tun::builder()
        .name("")
        .up()
        .try_build()
        .unwrap()
}

async fn connect_socket(args: Args) -> io::Result<UdpSocket> {
    let socket = UdpSocket::bind(SocketAddr::from(([0, 0, 0, 0], args.local_port))).await?; // TODO: listen on IPv6 as well
    socket.connect((args.remote_host, args.remote_port)).await?;
    Ok(socket)
}

async fn handle_outgoing(mut tun_reader: ReadHalf<Tun>, socket: Arc<UdpSocket>) {
    let mut buffer = vec![0u8; 1500];
    loop {
        match tun_reader.read(&mut buffer).await {
            Ok(n)  if n > 0 => {
                if let Err(e) = socket.send(&buffer[..n]).await {
                    eprintln!("Error sending packet to remote peer: {}", e);
                }
            },
            Ok(_) => continue,
            Err(e) => {
                    eprintln!("Error reading from tun device: {}", e);
                    break;
            }
        }
    }
}

async fn handle_incoming(mut tun_writer: WriteHalf<Tun>, socket:Arc<UdpSocket>) {
    let mut buffer = vec![0u8; 1500];
        loop {
            match socket.recv(&mut buffer).await {
                Ok(n) if n > 0 => {
                    if let Err(e) = tun_writer.write_all(&buffer[..n]).await {
                        eprintln!("Error writing incoming packet to tun device: {}", e)
                    }
                }
                Ok(_) => continue,
                Err(e) => {
                    eprintln!("Error receiving UDP packet: {}", e);
                    break;
                }
            }
        }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let tun = init_tun();

    let sock = Arc::new(connect_socket(args).await.unwrap());

    let (tun_reader, tun_writer) = tokio::io::split(tun);

    let outgoing = tokio::spawn(handle_outgoing(tun_reader, sock.clone()));

    let incoming = tokio::spawn(handle_incoming(tun_writer, sock));

    tokio::try_join!(outgoing, incoming)?;

    Ok(())
}
