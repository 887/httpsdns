use std::rc::Rc;

use std::io::{Error};
use futures::{Async, Poll};
use tokio_core::net::UdpSocket;

use futures::stream::Stream;

use types::{Request, log};

pub struct SocketRead {
    socket: Rc<UdpSocket>
}

impl SocketRead {
    pub fn new(socket: Rc<UdpSocket>) -> Self {
        SocketRead {
            socket: socket
        }
    }
}

impl Stream for SocketRead {
    type Item = Request;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        log("socket read polling..");
        if let Async::NotReady = self.socket.poll_read() {
            log("socket read not ready!");
            return Ok(Async::NotReady)
        }
        let mut buffer = [0; 1500];
        //this macro also handled the WouldBlock case,
        //but its easier to understand what happens here without it
        //let (amt, addr) = try_nb!(self.socket.recv_from(&mut buffer));
        match self.socket.recv_from(&mut buffer) {
            Ok((amt, addr)) => {
                log("socket read!");
                Ok(Async::Ready(Some((buffer, amt, addr))))
            },
            Err(ref e) if e.kind() == ::std::io::ErrorKind::WouldBlock => {
                log("socket read would block!");
                Ok(Async::NotReady)
            }
            Err(e) => {
                //socket closed?
                log("socket error!");
                Err(e)
            },
        }
    }
}
