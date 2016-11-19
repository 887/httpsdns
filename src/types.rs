// serde has serializing and deserializing implementations for SocketAddr
// https://lifthrasiir.github.io/rust-chrono/serde/ser/trait.Serialize.html
// use std::net::{SocketAddr, ToSocketAddrs};
use std::net::SocketAddr;
use tokio_core::net::UdpSocket;

use std::sync::Arc;

pub struct Receiver {
    pub socket: Arc<UdpSocket>,
    pub addr: SocketAddr,
}
pub struct ParsedPacket {
    pub id: u16,
}
pub type ReceiverRef = Arc<(Receiver)>;

pub type Buffer = [u8; 1500];
