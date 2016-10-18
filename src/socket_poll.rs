use std::io::{Error};
use std::net::SocketAddr;
use futures::{Async, Poll};
use tokio_core::net::UdpSocket;

use futures::stream::Stream;

use chrono::{Local};

use types::*;

pub struct SocketPoll<'a> {
    socket: &'a UdpSocket
}

impl<'a> SocketPoll<'a> {
    pub fn new(socket: &'a UdpSocket) -> Self {
        SocketPoll {
            socket: socket
        }
    }
}

//reading the docs really helped!
//i needed stream instead of future!
impl<'a> Stream for SocketPoll<'a> {
    type Item = Request;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        log("socket polling..");
        if let Async::NotReady = self.socket.poll_read() {
            log("socket not ready!");
            return Ok(Async::NotReady)
        }
        log("socket ready!");
        let mut buffer = [0; 1500];
        self.socket.recv_from(&mut buffer)
            .and_then(|(amt, addr)| Ok(Async::Ready(Some((buffer, amt, addr)))))
            //important: this not ready here is what keeps our server alive
            //(if there is an error or no data to read we just wait until there is more)
            //Ok(Async::Ready(None)), //Err(Error::new(ErrorKind::Other, "wrong read"))
            .or_else(|_|Ok(Async::NotReady))
    }
}

fn log(text: &str) {
    println!("{}: {}", Local::now().format("%Y-%m-%d %H:%M:%S").to_string(), text);
}

