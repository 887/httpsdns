use futures::{Async, Future, Poll};

use types::{Buffer, Request, ReceiverRef, log};

pub struct RequestResolver {
    receiver: ReceiverRef,
    buffer: Buffer,
    amt: usize,
}

impl RequestResolver {
    pub fn new((receiver, buffer, amt): Request) -> Self {
        RequestResolver {
            receiver: receiver,
            buffer: buffer,
            amt: amt,
        }
    }
}

impl Future for RequestResolver {
    type Item = Request;
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        log("resolving request");
        let receiver_ref = self.receiver.clone();
        Ok(Async::Ready((receiver_ref, self.buffer, self.amt)))
    }
}

