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
use std::net::{SocketAddr, ToSocketAddrs};

use std::sync::Arc;

use tokio_core::net::UdpSocket;
use tokio_core::reactor::Core;

use futures::*;
use futures::stream::Stream;
use futures_cpupool::CpuPool;

use tokio_core::net::TcpStream;

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
// sudo watch -n 5 "nmap -P0 -sU -p54321 127.0.0.1"

fn main() {
    let addr = env::args().nth(1).unwrap_or("127.0.0.1:54321".to_string());
    let addr = addr.parse::<SocketAddr>().unwrap();

    let mut core = Core::new().unwrap();
    let handle = core.handle();

    //https://developers.google.com/speed/public-dns/docs/dns-over-https
    //let config = Arc::new(Config{
        //addr: "dns.google.com:443".to_socket_addrs().unwrap().next().unwrap()
    //});

    //TODO: readconfigfile if exists -> config, else -> defaultconfig
    let config = Arc::new(Config{
        addr: "www.rust-lang.org:443".to_socket_addrs().unwrap().next().unwrap(),
        pool: 4,
    });

    let pool = CpuPool::new(config.pool);

    let socket = UdpSocket::bind(&addr, &handle).unwrap();

    let requests = SocketReader::new(socket);

    let answer_attempts = requests.map(|(receiver_ref, buffer, amt)| {
        RequestResolver::new(config.clone(), receiver_ref.clone(), buffer, amt)
            .and_then(|(receiver, request_string): (ReceiverRef, String)| {
                //let buffer = [0; 1500];
                //let amt = 0;
                //SocketSender::new((receiver, buffer, amt))

                //spawning one thread (or process if its pthreads & epool)?
                //per tcp connection with its own eventloop could be correct..
                let mut core = Core::new().unwrap();
                let handle = core.handle();
                let addr = "www.rust-lang.org:443".to_socket_addrs().unwrap().next().unwrap();
                let stream = TcpStream::connect(&addr, &handle);

                //now for the rest of this:
                //https://github.com/alexcrichton/futures-rs/blob/master/TUTORIAL.md#stream-example

                //i need to feed this context fresh tcp streams:
                //https://tokio-rs.github.io/tokio-tls/tokio_tls/struct.ClientContext.html
                //let c let tls_handshake = socket.and_then(|socket| {
                    //let cx = ClientContext::new().unwrap();
                    //cx.handshake("dns.google.com", socket)
                //});lient_context = ClientContext::new().unrwap();

                //TODO return a future to make this compile!
                //we still have the result from our last future in this context
                //and only need to chain it together into the tcp future to make this work
                //(in theory)

                let tls_handshake = stream.and_then(|socket| {
                    let cx = ClientContext::new().unwrap();
                    cx.handshake("www.rust-lang.org", socket)
                });
                //need to mock this, only temporary for testing if this works!
                //don't spam rust-lang.org!!
                let request = tls_handshake.and_then(|socket| {
                    tokio_core::io::write_all(socket, "\
                                              GET / HTTP/1.0\r\n\
                                              Host: www.rust-lang.org\r\n\
                                              \r\n\
                                              ".as_bytes())
                });
                let response = request.and_then(|(socket, _)| {
                    tokio_core::io::read_to_end(socket, Vec::new()).boxed()
                });
                match core.run(response) {
                    Ok((_, data)) => {
                        log(&format!("{} bytes read!", data.len()));
                        let mut arr = [0u8; 1500];
                        let mut len = 1500;
                        if data.len() < 1500 {
                            len = data.len();
                        }
                        for i in 0..len {
                            arr[i] = data[i];
                        }
                        SocketSender::new((receiver, arr, len)).boxed()
                    },
                    Err(_) => {
                        finished::<(), ()>(()).boxed()
                    }
                }
            })
    });

    let server = answer_attempts.for_each(|answer| {
        handle.spawn(pool.spawn(answer));
        Ok(())
    });

    core.run(server).unwrap();
}
