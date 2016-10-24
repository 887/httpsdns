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
        log("socket send polling..");
        if let Async::NotReady = self.receiver.socket.poll_write() {
            log("socket not ready!");
            return Ok(Async::NotReady);
        }
        match self.receiver.socket.send_to(&self.buffer, &self.receiver.addr) {
            Ok(amt) => {
                if amt < self.buffer.len() {
                    log("socket hasn't send enough!");
                    // try again maybe?
                    // Ok(Async::NotReady)
                    // its safer to drop it should this happen,
                    // because this could loop for ever
                    Ok(Async::Ready(()))
                } else {
                    log("socket send complete!");
                    Ok(Async::Ready(()))
                }
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                log("socket read would block!");
                Ok(Async::NotReady)
            }
            Err(_) => {
                // socket closed?
                log("socket error!");
                Err(())
            }
        }
    }
}
