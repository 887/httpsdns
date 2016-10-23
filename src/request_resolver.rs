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
    type Item = (ReceiverRef, Vec<Question>);
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        log("resolving request");

        let mock_questsions = vec![Question{
            qname: String::from("google.com"),
            qtype: 1,
        }];
        return Ok(Async::Ready((self.receiver.clone(), mock_questsions)));

        //TODO:
        //https://tailhook.github.io/dns-parser/dns_parser/struct.Packet.html
        if let Ok(packet) = Packet::parse(&self.buffer[..self.amt]) {
            //TODO turn this packet into a request string/construct for the api and handle the stream stuff
            //in the next futrue
            log(&format!("packet parsed! ({},{},{})",
                packet.questions.len(),
                packet.answers.len(),
                packet.nameservers.len()));
            let mut dns_questions = Vec::<Question>::new();
            for question in packet.questions {
                dns_questions.push(Question{
                    qname: String::from("google.com"),
                    qtype: 1,
                });
            }
            Ok(Async::Ready((self.receiver.clone(), dns_questions)))
        } else {
            Err(())
        }
    }
}

