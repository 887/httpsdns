use std::cell::RefCell;
use std::rc::Rc;

use std::io::{Error};
use std::net::SocketAddr;
use futures::{Async, Poll};
use tokio_core::net::UdpSocket;

use futures::stream::Stream;

use chrono::{Local};

use types::*;

pub struct SocketPoll {
    socket: Rc<UdpSocket>
}

impl SocketPoll {
    pub fn new(socket: Rc<UdpSocket>) -> Self {
        SocketPoll {
            socket: socket
        }
    }
}

//reading the docs really helped!
//i needed stream instead of future!
impl Stream for SocketPoll {
    type Item = Request;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        log("socket read polling..");
        if let Async::NotReady = self.socket.poll_read() {
            log("socket read not ready!");
            return Ok(Async::NotReady)
        }
        let mut buffer = [0; 1500];
        //this macro also handled the WouldBlock case (important!)
        let (amt, addr) = try_nb!(self.socket.recv_from(&mut buffer));
        log("socket read!");
        Ok(Async::Ready(Some((buffer, amt, addr))))
    }
}

fn log(text: &str) {
    println!("{}: {}", Local::now().format("%Y-%m-%d %H:%M:%S").to_string(), text);
}

