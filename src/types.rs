use std::net::SocketAddr;

pub type Buffer = [u8; 1500];
pub type Request = (Buffer, usize, SocketAddr);

