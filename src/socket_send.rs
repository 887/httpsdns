use std::io::{Error, ErrorKind};

use futures::{Async, Future, Poll, BoxFuture};

use chrono::{Local};

use tokio_core::net::UdpSocket;
use std::net::SocketAddr;

type DnsAnswer = String;

pub struct SocketDnsRequest {
    buffer: [u8; 1500],
    amt: usize,
    addr: SocketAddr,
}

impl SocketDnsRequest {
    pub fn new(buffer: [u8; 1500], amt: usize, addr: SocketAddr) -> Self {
        SocketDnsRequest {
            buffer: buffer,
            amt: amt,
            addr: addr,
        }
    }
}

impl Future for SocketDnsRequest {
    type Item = DnsAnswer;
    type Error = Error;

    fn poll(&mut self) -> Poll<DnsAnswer, Self::Error> {
        //TODO parse dns request
        //TODO do https web request to get DNS data
        Ok((Async::Ready("answer".to_string())))
    }
}

fn log(text: &str) {
    println!("{}: {}", Local::now().to_string(), text);
}
