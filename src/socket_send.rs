use std::io::ErrorKind;

use futures::{Async, Future, Poll};

use types::*;

pub struct SocketSender {
    receiver: ReceiverRef,
    buffer: Vec<u8>,
}

impl SocketSender {
    pub fn new((receiver, buffer): (ReceiverRef, Vec<u8>)) -> Self {
        SocketSender {
            receiver: receiver,
            buffer: buffer,
        }
    }
}

impl Future for SocketSender {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        debug!("socket send polling..");
        if let Async::NotReady = self.receiver.socket.poll_write() {
            debug!("socket not ready!");
            return Ok(Async::NotReady);
        }
        match self.receiver.socket.send_to(&self.buffer, &self.receiver.addr) {
            Ok(amt) => {
                if amt < self.buffer.len() {
                    debug!("socket hasn't send enough!");
                    // try again maybe?
                    // Ok(Async::NotReady)
                    // its safer to drop it should this happen,
                    // because this could loop for ever
                    Ok(Async::Ready(()))
                } else {
                    debug!("socket send complete!");
                    Ok(Async::Ready(()))
                }
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                debug!("socket read would block!");
                Ok(Async::NotReady)
            }
            Err(_) => {
                // socket closed?
                error!("socket error!");
                Err(())
            }
        }
    }
}
