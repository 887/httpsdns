//ok.. this is what i need as a base!:
//udp_echo_server_tokio.rs
//https://gist.github.com/hrektts/92f6a1afc31c9a3ed5d1ec3d5f91cd9e

//oh.. there is an example of how to build a resolving client with futures?
//https://github.com/alexcrichton/futures-trustdns-test/blob/5ebd74bfa041923bd2b44b14cc818f5511b80767/src/main.rs
//maybe i can turn this into a server.. somehow

//Reading list:
//https://github.com/iorust/futures-rs
//https://github.com/alexcrichton/futures-rs/blob/master/TUTORIAL.md#the-future-trait

//TODO: FIRST ORDER OF BUISNESS: println! on async ready and espacially! async::not ready to see
//how this eventloop does its thingy! (does it stop on an io read like libuv? whats going on
//here?!!?!
//
//huh interesting according to
//fn _run(&mut self, done: &mut FnMut() -> bool) {
//this is epoll in linux but i don't know if it will be iocp in windows
//impotant: still test the behavior as described on the TODO above

extern crate dns_parser;
extern crate toml;
extern crate futures;
extern crate futures_cpupool;
#[macro_use]
extern crate tokio_core;

use std::sync::mpsc::channel;

use std::env;
use std::io::{Error, ErrorKind};
use std::net::SocketAddr;

use futures::{Async, Future, Poll, BoxFuture};
use tokio_core::net::UdpSocket;
use tokio_core::reactor::Core;

use futures_cpupool::CpuPool;

//test udp port
//https://wiki.itadmins.net/network/tcp_udp_ping
//sudo watch -n 5 "nmap -P0 -sU -p8080 127.0.0.1"

struct Echo {
    socket: UdpSocket,
    buffer: [u8; 1500],
}

impl Echo {
    fn new(socket: UdpSocket) -> Self { Echo { socket: socket, buffer: [0; 1500] } }
}

impl Future for Echo {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<(), Self::Error> {
        loop {
            let socket_poll_result = self.socket.poll_read();
            println!("socket polled!");
            match socket_poll_result {
                Async::Ready(_) => {
                    println!("socket ready!");
                    //TODO: we should probably model the recv_from & answer as a future
                    //and push it on the CPU pool
                    let (amt, addr) = try_nb!(self.socket.recv_from(&mut self.buffer));
                    println!("socket data received!");
                    if 0 == amt {
                        //return Err(Error::new(ErrorKind::Other, "wrong read"));
                        let mock: [u8; 3] = [1,2,3];
                        let amt = try_nb!(self.socket.send_to(&mock, &addr));
                        if 0 == amt {
                            return Err(Error::new(ErrorKind::Other, "wrong write"));
                        } else {
                            println!("socket answer mock send!");
                        }
                    } else {
                        let amt = try_nb!(self.socket.send_to(&self.buffer[..amt], &addr));
                        if 0 == amt {
                            return Err(Error::new(ErrorKind::Other, "wrong write"));
                        } else {
                            println!("socket answer echoed!");
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
                    //ok this is pretty much whats needed here
                    println!("socket not ready!");
                    return Ok(Async::NotReady)
                },
            }
        }
    }
}

fn main() {
    let addr = env::args().nth(1).unwrap_or("127.0.0.1:8080".to_string());
    let addr = addr.parse::<SocketAddr>().unwrap();

    let mut l = Core::new().unwrap();
    let handle = l.handle();

    let socket = UdpSocket::bind(&addr, &handle).unwrap();
    println!("Listening on: {}", addr);

    let pool = CpuPool::new(4);

    let echo = Echo::new(socket);
    l.run(echo).unwrap();
}

























//extern crate dns_parser;
//extern crate toml;
//extern crate futures;
//extern crate tokio_core;
////extern crate tokio_proto;
//extern crate dns_parser;

//use std::env;
//use std::net::SocketAddr;

//use futures::Future;
//use futures::stream::Stream;
//use tokio_core::io::{copy, Io};
//use tokio_core::net::{TcpListener, UdpSocket};
//use tokio_core::reactor::Core;

// https://tokio-rs.github.io/tokio-core/tokio_core/index.html
// the tokio-rs example could make a solid foundation, need to investigate it more


//fn main() {
//let tcp_addr = env::args().nth(1).unwrap_or("127.0.0.1:8080".to_string());
//let udp_addr = env::args().nth(2).unwrap_or("127.0.0.1:8082".to_string());

//let tcp_addr = tcp_addr.parse::<SocketAddr>().unwrap();
//let udp_addr = udp_addr.parse::<SocketAddr>().unwrap();

//// Create the event loop that will drive this server
//let mut l = Core::new().unwrap();

//let tcp_handle = l.handle();
//let udp_handle = l.handle();

//let tcp_socket = TcpListener::bind(&tcp_addr, &tcp_handle).unwrap();
//let udp_socket = UdpSocket::bind(&udp_addr, &udp_handle).unwrap();

//println!("tcp listening on: {}", tcp_addr);
//println!("udp listening on: {}", udp_addr);

//let tcp_socket_handler = tcp_socket.incoming().for_each(|(socket, addr)| {
//let pair = futures::lazy(|| Ok(socket.split()));
////copy data from reading to writing half
//let amt = pair.and_then(|(reader, writer)| copy(reader, writer));
////spawn the future to allow it to run concurrently
//tcp_handle.spawn(amt.then(move |result| {
//println!("wrote {:?} bytes to {}", result, addr);
//Ok(())
//}));
//Ok(())
//});
//// Execute our server (modeled as a future) and wait for it to
//// complete.
//l.run(tcp_socket_handler).unwrap();

//TODO:
//there seems to be no tcp incoming counterpart for udp yet, and I probably shouldn't
//do actual socket reading/writing operations on the same thread I am listening for
//connections.
//here is an example with the threadpool api, i probably need this:
//https://docs.rs/futures-cpupool/0.1.2/futures_cpupool/
//also if I want to involve hyper later to parse the google https dns api and
//i also may need futures for that too?
//here is a more or less generic future example:
//http://alexcrichton.com/futures-rs/futures/index.html

//well turns out i don't undertand UDP at all
//https://doc.rust-lang.org/std/net/struct.UdpSocket.html
//Taken from the description at docs.rustlang:
//A User Datagram Protocol socket.
//This is an implementation of a bound UDP socket.

// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
//This supports both IPv4 and IPv6 addresses,
//and there is no corresponding notion of a server because UDP is a datagram protocol.
// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

//use std::net::UdpSocket;
//{
//let mut socket = try!(UdpSocket::bind("127.0.0.1:34254"));

//// read from the socket
//let mut buf = [0; 10];
//let (amt, src) = try!(socket.recv_from(&mut buf));

//// send a reply to the socket we received data from
//let buf = &mut buf[..amt];
//buf.reverse();
//try!(socket.send_to(buf, &src));
//} // the socket is closed here

//Ok. That means no connections and i need to choose a buffer thats up to specs..
//more reading to do till weekend

//oh.. there is an example of how to build a resolving client with futures?
//https://github.com/alexcrichton/futures-trustdns-test/blob/5ebd74bfa041923bd2b44b14cc818f5511b80767/src/main.rs
//maybe i can turn this into a server.. somehow

//Reading list:
//https://github.com/iorust/futures-rs
//https://github.com/alexcrichton/futures-rs/blob/master/TUTORIAL.md#the-future-trait

//maybe do the server via the line crate?
//https://github.com/tokio-rs/tokio-line/blob/master/examples/echo_client_server.rs

//ok this + example above+ udp from tokio_core should give me what i need
//https://github.com/tokio-rs/tokio-line/blob/master/src/service.rs

//ok aparently we need the "server" from "tokio-proto" -> deeper down the rabbit hole we go
//https://github.com/tokio-rs/tokio-line/blob/master/src/lib.rs
//// The `tokio_proto` crate contains the abstractions and building blocks for
//// quickly implementing a protocol client or server.
//extern crate tokio_proto as proto;

//more resources: http://stackoverflow.com/questions/39049365/rust-echo-server-and-client-using-futures-blocks-itself-forever
//(build a client to test it?)

//a combination of these two.. nope these are still tcp, what am i stupposed to do here?!
//https://github.com/tokio-rs/tokio-line/blob/master/src/service.rs
//https://github.com/tokio-rs/tokio-line/blob/master/examples/echo_client_server.rs

//}
