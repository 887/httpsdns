use chrono::{Local};

use std::net::SocketAddr;

pub type Buffer = [u8; 1500];
pub type Request = (Buffer, usize, SocketAddr);

pub fn log(text: &str) {
    println!("{}: {}", Local::now().format("%Y-%m-%d %H:%M:%S").to_string(), text);
}
