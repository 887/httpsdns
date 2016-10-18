extern crate dns_parser;
extern crate toml;
extern crate futures;
extern crate futures_cpupool;
extern crate chrono;

#[macro_use]
extern crate tokio_core;

use std::io::{Error, ErrorKind};
//use futures::Future;

use std::env;
use std::net::SocketAddr;

use tokio_core::net::UdpSocket;
use tokio_core::reactor::Core;

use futures::stream::Stream;

use futures_cpupool::CpuPool;

mod types;
mod socket_poll;
use socket_poll::*;
mod socket_send;
use socket_send::*;


//test udp port
//https://wiki.itadmins.net/network/tcp_udp_ping
//sudo watch -n 5 "nmap -P0 -sU -p54321 127.0.0.1"

//examples that used cpupool + futures
//https://github.com/tailhook/abstract-ns/blob/8e28eb934a3ffe2e9f64134e34afd422a1810f88/examples/routing.rs

//omg does this exist? //nope but its another example for cpupool (and also a TCP addr name
//resolver)
//https://github.com/sbstp/tokio-dns

fn main() {
    let addr = env::args().nth(1).unwrap_or("127.0.0.1:54321".to_string());
    let addr = addr.parse::<SocketAddr>().unwrap();

    let mut l = Core::new().unwrap();
    let handle = l.handle();

    //probably best to wrap this in a RefCell so it can be accessed from multiple threads later
    let socket = UdpSocket::bind(&addr, &handle).unwrap();
    println!("Listening on: {}", addr);

    //hellspawn i choose you!
    let pool = CpuPool::new(4);

    //rename to SocketPollRead
    let requests = SocketPoll::new(&socket);

    //this is still sync but works
    ////the magic 'loop' that keeps this alive is the for_each and only exists if you use Stream!
    ////i didn't understand this at all!
    ////TODO and_then(turn_into_dns_request and resolve_https_dns_request_future).for_each
    //let server = requests.for_each(|(buffer, amt, addr)|{
    //match socket.send_to(&buffer[..amt], &addr) {
    //Ok(amt) => Ok(()),
    //_ => Err(Error::new(ErrorKind::Other, "wrong write")),
    //}
    ////Ok(())
    //});

    //taken from this example..
    //https://github.com/alexcrichton/futures-rs/blob/master/TUTORIAL.md#stream-example
    let request_answered_futures = requests.map(|(buffer, amt, addr)| {
        //i guess iam also missing poll_write to get true concurrency?
        //TODO need to construct a future with poll write here and return it! (should be a future
        //not a stream!)
        //IMPORTANT: this future should not return async::not_ready on error but instead
        //return a None to indicate that the answer didn't work but the server is allowed to
        //continue to operate
        SocketSend::new(&socket, (buffer, amt, addr))
    });
    //TODO: also construct the dns resolve as a mapped future

    let server = request_answered_futures.for_each(|write_future| {
        handle.spawn(write_future);
        Ok(())
    });

    l.run(server).unwrap();
}
