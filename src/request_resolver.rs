use futures::{Async, Future, Poll};

use std::sync::Arc;

use types::*;

use dns_parser::{Packet, QueryType};

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

        let mock_questions = vec![Question{
            qname: String::from("google.com"),
            qtype: 1,
        }];
        return Ok(Async::Ready((self.receiver.clone(), mock_questions)));

        //TODO:
        //https://tailhook.github.io/dns-parser/dns_parser/struct.Packet.html
        if let Ok(packet) = Packet::parse(&self.buffer[..self.amt]) {
            log(&format!("packet parsed! ({},{},{})",
                packet.questions.len(),
                packet.answers.len(),
                packet.nameservers.len()));

            //only support question for now
            //https://groups.google.com/forum/#!topic/comp.protocols.dns.bind/uOWxNkm7AVg
            if packet.questions.len() != 1 {
                return Err(())
            }

            let mut dns_questions = Vec::<Question>::new();
            for question in packet.questions {
                //only support those that the google API supports
                let qtype = match question.qtype {
                    QueryType::A => 1,
                    QueryType::AAAA => 28,
                    QueryType::CNAME => 5,
                    QueryType::MX => 15,
                    QueryType::All => 255, //ANY
                    _ => return Err(())
                };
                dns_questions.push(Question{
                    qname: question.qname.to_string(),
                    qtype: qtype,
                });
            }
            Ok(Async::Ready((self.receiver.clone(), dns_questions)))
        } else {
            Err(())
        }
    }
}

