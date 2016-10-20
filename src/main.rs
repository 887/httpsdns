extern crate dns_parser;
extern crate toml;
extern crate futures;
extern crate chrono;
extern crate futures_cpupool;

//cool!
//https://github.com/tokio-rs/tokio-tls/blob/master/Cargo.toml
extern crate tokio_tls;

#[macro_use]
extern crate tokio_core;

use std::env;
use std::net::SocketAddr;

use tokio_core::net::UdpSocket;
use tokio_core::reactor::Core;

use futures::Future;
use futures::stream::Stream;
use futures_cpupool::CpuPool;

mod types;
mod socket_read;
use socket_read::*;
mod socket_send;
use socket_send::*;
mod request_resolver;
use request_resolver::*;

// test udp port
// https://wiki.itadmins.net/network/tcp_udp_ping
// sudo watch -n 0.1 "nmap -P0 -sU -p54321 127.0.0.1"

fn main() {
    let addr = env::args().nth(1).unwrap_or("127.0.0.1:54321".to_string());
    let addr = addr.parse::<SocketAddr>().unwrap();

    let mut l = Core::new().unwrap();
    let handle = l.handle();

    let socket = UdpSocket::bind(&addr, &handle).unwrap();

    let requests = SocketReader::new(socket);

    let answer_attempts = requests.map(|request| {
        RequestResolver::new(request).and_then(SocketSender::new)
    });

    let pool = CpuPool::new(4);

    let server = answer_attempts.for_each(|answer| {
        handle.spawn(pool.spawn(answer));
        Ok(())
    });

    l.run(server).unwrap();
}
