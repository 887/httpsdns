extern crate dns_parser;
extern crate toml;
extern crate futures;
extern crate chrono;
extern crate futures_cpupool;

#[macro_use]
extern crate tokio_core;

use std::env;
use std::net::SocketAddr;

use tokio_core::net::UdpSocket;
use tokio_core::reactor::Core;

use futures::stream::Stream;
use futures_cpupool::CpuPool;

mod types;
mod socket_read;
use socket_read::*;
mod socket_send;
use socket_send::*;

//test udp port
//https://wiki.itadmins.net/network/tcp_udp_ping
//sudo watch -n 0.1 "nmap -P0 -sU -p54321 127.0.0.1"

fn main() {
    let addr = env::args().nth(1).unwrap_or("127.0.0.1:54321".to_string());
    let addr = addr.parse::<SocketAddr>().unwrap();

    let mut l = Core::new().unwrap();
    let handle = l.handle();

    let socket = UdpSocket::bind(&addr, &handle).unwrap();

    let requests = SocketReader::new(socket);

    let request_answered_futures = requests.map(|request| {
        SocketSender::new(request)
    });

    let pool = CpuPool::new(4);

    //TODO: also construct the dns resolve as a mapped future
    // .requests
    // .map(|(buffer, amt, addr)| { turn_into_dns_request and resolve_https_dns_request_future})
    // .map(|(buffer, amt, addr)| { SocketSend::new(socket.clone(), (buffer, amt, addr)) })
    // .foreach as seen below but with cpupool?
    // (maybe do not use the cpu for now, as its not necessary)
    let server = request_answered_futures.for_each(|write_future| {
        //this should make this run in async
        //https://github.com/alexcrichton/futures-rs/blob/master/TUTORIAL.md#stream-example
        handle.spawn(pool.spawn(write_future));
        Ok(())
    });

    l.run(server).unwrap();
}
