use futures::{Async, Future, Poll};

use std::sync::Arc;

use types::*;

use dns_parser::Packet;

pub struct RequestResolver {
    config: Arc<Config>,
    receiver: ReceiverRef,
    buffer: Buffer,
    amt: usize,
}

impl RequestResolver {
    pub fn new(config: Arc<Config>, receiver: ReceiverRef, buffer: Buffer, amt: usize) -> Self {
        RequestResolver {
            config: config,
            receiver: receiver,
            buffer: buffer,
            amt: amt,
        }
    }
}

impl Future for RequestResolver {
    type Item = (ReceiverRef, String);
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        log("resolving request");

        return Ok(Async::Ready((self.receiver.clone(), "google.com".to_string())));

        //TODO:
        //https://tailhook.github.io/dns-parser/dns_parser/struct.Packet.html
        if let Ok(packet) = Packet::parse(&self.buffer[..self.amt]) {
            //TODO turn this packet into a request string/construct for the api and handle the stream stuff
            //in the next futrue
            log(&format!("packet parsed! ({},{},{})",
                packet.questions.len(),
                packet.answers.len(),
                packet.nameservers.len()));
            Ok(Async::Ready((self.receiver.clone(), "google.com".to_string())))
        } else {
            Err(())
        }
    }
}

