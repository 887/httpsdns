use std::io::{Error, ErrorKind};

use std::sync::Arc;

use futures::{Async, Poll};
use futures::stream::Stream;

use tokio_core::net::UdpSocket;

use types::*;

pub struct SocketReader {
    socket: Arc<UdpSocket>,
}

impl SocketReader {
    pub fn new(socket: UdpSocket) -> Self {
        SocketReader { socket: Arc::new(socket) }
    }
}

impl Stream for SocketReader {
    type Item = (ReceiverRef, Buffer, usize);
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        debug!("socket read polling..");
        if let Async::NotReady = self.socket.poll_read() {
            debug!("socket read not ready!");
            return Ok(Async::NotReady);
        }
        let mut buffer = [0; 1500];
        // this macro also handled the WouldBlock case,
        // but its easier to understand what happens here without it
        // let (amt, addr) = try_nb!(self.socket.recv_from(&mut buffer));
        match self.socket.recv_from(&mut buffer) {
            Ok((amt, addr)) => {
                debug!("socket read!");
                let socket_ref = Arc::new(Receiver {
                    socket: self.socket.clone(),
                    addr: addr,
                });
                Ok(Async::Ready(Some((socket_ref, buffer, amt))))
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                debug!("socket read would block!");
                Ok(Async::NotReady)
            }
            Err(e) => {
                // socket closed?
                error!("socket error!");
                Err(e)
            }
        }
    }
}
