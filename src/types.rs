use chrono::{Local};

use std::net::SocketAddr;
use tokio_core::net::UdpSocket;

//use std::rc::Rc;
use std::sync::Arc;

pub type SocketRef = Arc<UdpSocket>;
pub type Buffer = [u8; 1500];
pub type Request = (SocketRef, Buffer, usize, SocketAddr);

pub fn log(text: &str) {
    println!("{}: {}", Local::now().format("%Y-%m-%d %H:%M:%S").to_string(), text);
}
