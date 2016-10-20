use futures::{Async, Future, Poll};

use types::{Buffer, Request, ReceiverRef, log};

use dns_parser::Packet;

//next up:
//tokio-tls = { git = "https://github.com/tokio-rs/tokio-tls" }

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
        //https://tailhook.github.io/dns-parser/dns_parser/struct.Packet.html
        if let Ok(packet) = Packet::parse(&self.buffer) {
            log(&format!("packet parsed! ({},{},{})",
                packet.questions.len(),
                packet.answers.len(),
                packet.nameservers.len()));
            Ok(Async::Ready((self.receiver.clone(), self.buffer, self.amt)))
        } else {
            Err(())
        }
    }
}

