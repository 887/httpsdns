use chrono::Local;

use std::net::SocketAddr;
use tokio_core::net::UdpSocket;

// use std::rc::Rc;
use std::sync::Arc;

pub struct Receiver {
    pub socket: Arc<UdpSocket>,
    pub addr: SocketAddr
}
pub type ReceiverRef = Arc<(Receiver)>;

pub struct Config{
    pub addr: SocketAddr
}

pub type Buffer = [u8; 1500];

pub fn log(text: &str) {
    println!("{}: {}",
             Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
             text);
}
