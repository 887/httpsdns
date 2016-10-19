use std::rc::Rc;

use futures::{Async, Future, Poll};

use tokio_core::net::UdpSocket;
use std::net::SocketAddr;

use types::{Buffer, Request, log};

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
        log("socket send polling..");
        if let Async::NotReady = self.socket.poll_write() {
            log("socket not ready!");
            return Ok(Async::NotReady)
        }
        match self.socket.send_to(&self.buffer[..self.amt], &self.addr) {
            Ok(amt) => {
                if amt < self.amt {
                    //try again maybe?
                    log("socket hasn't send enough!");
                    Ok(Async::NotReady)
                } else {
                    log("socket send complete!");
                    Ok(Async::Ready(()))
                }
            },
            Err(ref e) if e.kind() == ::std::io::ErrorKind::WouldBlock => {
                log("socket read would block!");
                Ok(Async::NotReady)
            }
            Err(_) => {
                //socket closed?
                log("socket error!");
                Err(())
            },
        }
    }
}
