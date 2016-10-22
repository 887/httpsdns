use std::io::{ErrorKind};

use futures::{Async, Future, Poll};

use types::*;

pub struct SocketSender {
    receiver: ReceiverRef,
    buffer: Buffer,
    amt: usize,
}

impl SocketSender {
    pub fn new((receiver, buffer, amt): (ReceiverRef, Buffer, usize)) -> Self {
        SocketSender {
            receiver: receiver,
            buffer: buffer,
            amt: amt,
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
            return Ok(Async::NotReady)
        }
        match self.receiver.socket.send_to(&self.buffer[..self.amt], &self.receiver.addr) {
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
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
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
