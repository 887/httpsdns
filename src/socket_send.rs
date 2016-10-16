use tokio_core::net::UdpSocket;
use std::net::SocketAddr;

pub struct SocketSend {
    socket: UdpSocket,
    buffer: [u8; 1500],
    adr: SocketAddr,
}

