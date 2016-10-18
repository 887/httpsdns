use std::rc::Rc;

use futures::{Async, Future, Poll};
use chrono::{Local};

use tokio_core::net::UdpSocket;
use std::net::SocketAddr;

use types::{Buffer, Request};

pub struct SocketSend {
    socket: Rc<UdpSocket>,
    buffer: Buffer,
    amt: usize,
    addr: SocketAddr,
}

impl SocketSend {
    pub fn new(socket: Rc<UdpSocket>, (buffer, amt, addr): Request) -> Self {
        SocketSend {
            socket: socket,
            buffer: buffer,
            amt: amt,
            addr: addr,
        }
    }
}

impl Future for SocketSend {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        log("socket write polling..");
        if let Async::NotReady = self.socket.poll_write() {
            log("socket not ready!");
            return Ok(Async::NotReady)
        }
        match self.socket.send_to(&self.buffer[..self.amt], &self.addr) {
            Ok(amt) => {
                if amt <= self.amt {
                    //written to little, try again maybe?
                    Ok(Async::NotReady)
                } else {
                    log("socket written!");
                    Ok(Async::Ready(()))
                }
            },
            Err(ref e) if e.kind() == ::std::io::ErrorKind::WouldBlock => {
                return Ok(Async::NotReady)
            }
            Err(_) => return Err(()),
        }
    }
}

fn log(text: &str) {
    println!("{}: {}", Local::now().format("%Y-%m-%d %H:%M:%S").to_string(), text);
}
