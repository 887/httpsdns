extern crate dns_parser;
extern crate toml;
extern crate futures;
extern crate futures_cpupool;
extern crate chrono;
#[macro_use]
extern crate tokio_core;

use std::env;
use std::net::SocketAddr;

use tokio_core::net::UdpSocket;
use tokio_core::reactor::Core;

use futures_cpupool::CpuPool;

mod socket_send;
mod socket_poll;
use socket_poll::*;

//test udp port
//https://wiki.itadmins.net/network/tcp_udp_ping
//sudo watch -n 5 "nmap -P0 -sU -p8080 127.0.0.1"

//good example that used cpupool + futures:
//https://github.com/tokio-rs/tokio-socks5/blob/27408359e46f6b263ece03bf206828952a49689f/src/main.rs

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
