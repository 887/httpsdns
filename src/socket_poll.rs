use std::sync::mpsc::channel;

use std::io::{Error, ErrorKind};
use std::net::SocketAddr;

use futures::{Async, Future, Poll, BoxFuture};
use tokio_core::net::UdpSocket;
use tokio_core::reactor::Core;

use chrono::{Local};
use futures_cpupool::CpuPool;

use socket_send::*;

type DnsAnswer = [u8; 1500];

pub struct SocketPoll {
    socket: UdpSocket,
    pool: CpuPool,
}

impl SocketPoll {
    pub fn new(socket: UdpSocket, pool: CpuPool) -> Self {
        SocketPoll {
            socket: socket,
            pool: pool,
        }
    }
}

impl Future for SocketPoll {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<(), Self::Error> {
        loop {
            log("socket polling..");
            let socket_poll_result = self.socket.poll_read();
            log("socket polled!");
            match socket_poll_result {
                Async::Ready(_) => {
                    log("socket ready!");

                    let mut buffer = [0; 1500];
                    let (amt, addr) = try_nb!(self.socket.recv_from(&mut buffer));

                    //TODO: make this part work
                    //let ss = SocketDnsRequest::new(buffer, amt, addr);
                    //match self.pool.spawn(ss) {
                        //Ok(answer) => {
                            //match self.socket.send_to(&answer, &addr) {
                              //Ok(amt) => {log("DNS request answered!");}
                              //Err(_) => {log("Couldn't answer DNS request");}
                            //}
                        //},
                        //_ => {}
                    //}
                    match self.socket.send_to(&buffer[..amt], &addr) {
                        Ok(_) => {log("DNS request answered!");}
                        Err(_) => {log("Couldn't answer DNS request");}
                    }

                },
                _ => {
                    log("socket not ready!");
                    return Ok(Async::NotReady)
                },
                }
            }
        }
    }

fn log(text: &str) {
    println!("{}: {}", Local::now().to_string(), text);
}


