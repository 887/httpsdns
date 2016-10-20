use futures::{Async, Future, Poll};

use std::sync::Arc;

use types::*;

use dns_parser::Packet;

use tokio_core::net::TcpStreamNew;
use tokio_tls::ClientContext;

pub struct RequestResolver {
    config: Arc<Config>,
    receiver: ReceiverRef,
    stream: TcpStreamNew,
    buffer: Buffer,
    amt: usize,
}

impl RequestResolver {
    pub fn new(config: Arc<Config>, receiver: ReceiverRef, stream: TcpStreamNew, buffer: Buffer, amt: usize) -> Self {
        RequestResolver {
            config: config,
            receiver: receiver,
            stream: stream,
            buffer: buffer,
            amt: amt,
        }
    }
}

impl Future for RequestResolver {
    type Item = (ReceiverRef, Buffer, usize);
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        log("resolving request");

        //finally got my tcpstreamnew here where i need it!

        //now for the rest of this:
        //https://github.com/alexcrichton/futures-rs/blob/master/TUTORIAL.md#stream-example

        //
        //https://tailhook.github.io/dns-parser/dns_parser/struct.Packet.html
        if let Ok(packet) = Packet::parse(&self.buffer) {

            //i need to feed this context fresh tcp streams:
            //https://tokio-rs.github.io/tokio-tls/tokio_tls/struct.ClientContext.html

            //if let client_context = ClientContext::new() {
                //client_context.handshake("dns.google.com", self.stream)
                //.and_then(a||

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

