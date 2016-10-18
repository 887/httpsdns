use std::io::{Error, ErrorKind};
use futures::{Async, Future, Poll, BoxFuture};
use chrono::{Local};

use tokio_core::net::UdpSocket;
use std::net::SocketAddr;

use types::{Buffer, Request};

pub struct SocketSend<'a> {
    socket: &'a UdpSocket,
    buffer: Buffer,
    amt: usize,
    addr: SocketAddr,
}

impl<'a> SocketSend<'a> {
    pub fn new(socket: &'a UdpSocket, (buffer, amt, addr): Request) -> Self {
        SocketSend {
            socket: socket,
            buffer: buffer,
            amt: amt,
            addr: addr,
        }
    }
}

impl<'a> Future for SocketSend<'a> {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        log("socket polling..");
        if let Async::NotReady = self.socket.poll_write() {
            log("socket not ready!");
            return Ok(Async::NotReady)
        }
        log("socket ready!");
        //match self.socket.send_to(&self.buffer[..self.amt], &self.addr) {
            //Ok(amt) => Ok(Async::Ready(amt)),
            //_ => Err(Error::new(ErrorKind::Other, "wrong write")),
        //}
        self.socket.send_to(&self.buffer[..self.amt], &self.addr)
            //the request is done now, regardless if send was sucessfull or not
            .and_then(|_| Ok(Async::Ready(())))
            .or_else(|_| Ok(Async::Ready(())))
    }
}

fn log(text: &str) {
    println!("{}: {}", Local::now().format("%Y-%m-%d %H:%M:%S").to_string(), text);
}
