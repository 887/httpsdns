use std::sync::mpsc::channel;

use std::io::{Error, ErrorKind};
use std::net::SocketAddr;

use futures::{Async, Future, Poll, BoxFuture};
use tokio_core::net::UdpSocket;
use tokio_core::reactor::Core;

use chrono::{Local};
use futures_cpupool::CpuPool;

use socket_send::*;

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
            let socket_poll_result = self.socket.poll_read();
            log("socket polled!");
            match socket_poll_result {
                Async::Ready(_) => {
                    log("socket ready!");

                    //the original design was to just have one buffer at runtime
                    //but i want to hand of this buffer to a different thread latter so
                    //it makes more sense to initialize it here so i can move its ownership later
                    let mut buffer = [0; 1500];
                    let (amt, addr) = try_nb!(self.socket.recv_from(&mut buffer));

                    //TODO: we should probably model the answer as a future
                    //and push it on the CPU pool
                    //let ss = SocketSend {
                    //socket: self.socket,
                    //buffer: buffer;
                    //adr: adr,
                    //};
                    //self.pool.spawn(SocketSend);
                    //pool.spawn(recv/answer);

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
                        let amt = try_nb!(self.socket.send_to(&buffer[..amt], &addr));
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


