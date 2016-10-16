extern crate dns_parser;
extern crate toml;
extern crate futures;
extern crate futures_cpupool;
extern crate chrono;
#[macro_use]
extern crate tokio_core;

use std::sync::mpsc::channel;

use std::env;
use std::io::{Error, ErrorKind};
use std::net::SocketAddr;

use futures::{Async, Future, Poll, BoxFuture};
use tokio_core::net::UdpSocket;
use tokio_core::reactor::Core;

use chrono::{Local};

use futures_cpupool::CpuPool;

//test udp port
//https://wiki.itadmins.net/network/tcp_udp_ping
//sudo watch -n 5 "nmap -P0 -sU -p8080 127.0.0.1"

struct SocketSend {
    socket: UdpSocket,
    buffer: [u8; 1500],
    adr: SocketAddr,
}

struct SocketPoll {
    socket: UdpSocket,
    pool: CpuPool,
    buffer: [u8; 1500],
}

impl SocketPoll {
    fn new(socket: UdpSocket, pool: CpuPool) -> Self {
        SocketPoll {
            socket: socket,
            pool: pool,
            buffer: [0; 1500]
        }
    }
}

impl Future for SocketPoll {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<(), Self::Error> {
        loop {
            let socket_poll_result = self.socket.poll_read();
            log("socket polled!");
            match socket_poll_result {
                Async::Ready(_) => {
                    log("socket ready!");
                    //TODO: we should probably model the recv_from or at least the answer as a future
                    //and push it on the CPU pool
                    //pool.spawn(recv/answer);

                    let (amt, addr) = try_nb!(self.socket.recv_from(&mut self.buffer));
                    log("socket data received!");
                    if 0 == amt {
                        //return Err(Error::new(ErrorKind::Other, "wrong read"));
                        let mock: [u8; 3] = [1,2,3];
                        let amt = try_nb!(self.socket.send_to(&mock, &addr));
                        if 0 == amt {
                            return Err(Error::new(ErrorKind::Other, "wrong write"));
                        } else {
                            log("socket answer mock send!");
                        }
                    } else {
                        let amt = try_nb!(self.socket.send_to(&self.buffer[..amt], &addr));
                        if 0 == amt {
                            return Err(Error::new(ErrorKind::Other, "wrong write"));
                        } else {
                            log("socket answer echoed!");
                        }
                    }
                },
                _ => {
                    //this only happens once!
                    //nice, no useless cpu cycles!
                    //it reads in the docs to this:
                    //If this function returns `Async::NotReady` then the current future's
                    //task is arranged to receive a notification when it might not return
                    //`NotReady`.
                    //
                    //ok this is pretty much whats needed here, although i have no idea WHY it
                    //works and maybe it depends on platform specific details in the background ?
                    //(linux epoll, windows iocp etc)
                    //huh interesting according to
                    //fn _run(&mut self, done: &mut FnMut() -> bool) {
                    //this is epoll in linux but i don't know if it will be iocp in windows
                    //impotant: still test the behavior as described on the TODO above
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

fn main() {
    let addr = env::args().nth(1).unwrap_or("127.0.0.1:8080".to_string());
    let addr = addr.parse::<SocketAddr>().unwrap();

    let mut l = Core::new().unwrap();
    let handle = l.handle();

    let socket = UdpSocket::bind(&addr, &handle).unwrap();
    println!("Listening on: {}", addr);

    let pool = CpuPool::new(4);

    let echo = SocketPoll::new(socket, pool);
    l.run(echo).unwrap();
}
