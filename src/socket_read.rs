use std::io::{Error, ErrorKind};

use std::sync::Arc;

use futures::{Async, Poll};
use futures::stream::Stream;

use tokio_core::net::UdpSocket;

use types::{Request, SocketRef, log};

pub struct SocketRead {
    socket: SocketRef
}

impl SocketRead {
    pub fn new(socket: UdpSocket) -> Self {
        SocketRead {
            socket: Arc::new(socket)
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
                let socket_ref = self.socket.clone();
                Ok(Async::Ready(Some((socket_ref, buffer, amt, addr))))
            },
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
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
