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

use std::sync::Arc;

use std::env;
use std::net::SocketAddr;

use tokio_core::net::UdpSocket;
use tokio_core::reactor::Core;

use futures::Future;
use futures::stream::Stream;
use futures_cpupool::CpuPool;

use tokio_core::net::TcpStream;
use tokio_core::net::TcpStreamNew;
use tokio_tls::ClientContext;

mod types;
mod socket_read;
use socket_read::*;
mod socket_send;
use socket_send::*;
mod request_resolver;
use request_resolver::*;

use types::*;

// test udp port
// https://wiki.itadmins.net/network/tcp_udp_ping
// sudo watch -n 0.1 "nmap -P0 -sU -p54321 127.0.0.1"

fn main() {
    let addr = env::args().nth(1).unwrap_or("127.0.0.1:54321".to_string());
    let addr = addr.parse::<SocketAddr>().unwrap();

    let mut l = Core::new().unwrap();
    let handle = l.handle();

    //https://developers.google.com/speed/public-dns/docs/dns-over-https
    let config = Arc::new(Config{addr: "dns.google.com:443".parse::<SocketAddr>().unwrap()});

    let socket = UdpSocket::bind(&addr, &handle).unwrap();

    let requests = SocketReader::new(socket);

    let answer_attempts = requests.map(|(receiver_ref, buffer, amt)| {
        RequestResolver::new(config.clone(), receiver_ref.clone(), stream, buffer, amt)
            .and_then(|(receiver, request_string): (ReceiverRef, String)| {
                let stream = TcpStream::connect(&config.addr, &handle);

                //now for the rest of this:
                //https://github.com/alexcrichton/futures-rs/blob/master/TUTORIAL.md#stream-example

                //i need to feed this context fresh tcp streams:
                //https://tokio-rs.github.io/tokio-tls/tokio_tls/struct.ClientContext.html
                //let client_context = ClientContext::new().unrwap();
                //client_context.handshake("dns.google.com", self.stream)
                //.and_then(a||


                //TODO return a future to make this compile!
                //we still have the result from our last future in this context
                //and only need to chain it together into the tcp future to make this work
                //(in theory)
            })
            .and_then(SocketSender::new)
    });

    let pool = CpuPool::new(4);

    let server = answer_attempts.for_each(|answer| {
        handle.spawn(pool.spawn(answer));
        Ok(())
    });

    l.run(server).unwrap();
}
