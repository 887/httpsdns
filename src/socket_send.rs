use std::io::{Error, ErrorKind};
use futures::{Async, Future, Poll, BoxFuture};
use chrono::{Local};

use tokio_core::net::UdpSocket;
use std::net::SocketAddr;

pub struct SocketSend<'a> {
    socket: &'a UdpSocket,
    buffer: [u8; 1500],
    amt: usize,
    addr: SocketAddr,
}

impl<'a> SocketSend<'a> {
    pub fn new(socket: &'a UdpSocket, buffer: [u8; 1500], amt: usize, addr: SocketAddr) -> Self {
        SocketSend {
            socket: socket,
            buffer: buffer,
            amt: amt,
            addr: addr,
        }
    }
}

impl<'a> Future for SocketSend<'a> {
    type Item = usize;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        TODO
    }
}

fn log(text: &str) {
    println!("{}: {}", Local::now().format("%Y-%m-%d %H:%M:%S").to_string(), text);
}
