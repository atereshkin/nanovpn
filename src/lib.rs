use std::io;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::UdpSocket;
use tokio_tun::Tun;

pub struct Tunnel {
    socket: Arc<UdpSocket>,
}

impl Tunnel {
    pub fn new(socket: UdpSocket) -> Self {
        Tunnel {
            socket: Arc::new(socket),
        }
    }

    pub async fn start(&self) -> io::Result<()> {
        let tun = Tunnel::init_tun();

        let (tun_reader, tun_writer) = tokio::io::split(tun);

        let outgoing = tokio::spawn(Tunnel::handle_outgoing(tun_reader, self.socket.clone()));

        let incoming = tokio::spawn(Tunnel::handle_incoming(tun_writer, self.socket.clone()));

        tokio::try_join!(outgoing, incoming)?;
        Ok(())
    }

    fn init_tun() -> Tun {
        Tun::builder().name("").up().try_build().unwrap()
    }

    async fn handle_outgoing(mut tun_reader: ReadHalf<Tun>, socket: Arc<UdpSocket>) {
        let mut buffer = vec![0u8; 1500];
        loop {
            match tun_reader.read(&mut buffer).await {
                Ok(n) if n > 0 => {
                    if let Err(e) = socket.send(&buffer[..n]).await {
                        eprintln!("Error sending packet to remote peer: {}", e);
                    }
                }
                Ok(_) => continue,
                Err(e) => {
                    eprintln!("Error reading from tun device: {}", e);
                    break;
                }
            }
        }
    }

    async fn handle_incoming(mut tun_writer: WriteHalf<Tun>, socket: Arc<UdpSocket>) {
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
}
