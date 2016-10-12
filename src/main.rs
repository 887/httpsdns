extern crate toml;
extern crate futures;
extern crate tokio_core;
use std::env;
use std::net::SocketAddr;

use futures::Future;
use futures::stream::Stream;
use tokio_core::io::{copy, Io};
use tokio_core::net::{TcpListener, UdpSocket};
use tokio_core::reactor::Core;

// https://tokio-rs.github.io/tokio-core/tokio_core/index.html
// the tokio-rs example could make a solid foundation, need to investigate it more

fn main() {
    let tcp_addr = env::args().nth(1).unwrap_or("127.0.0.1:8080".to_string());
    let udp_addr = env::args().nth(2).unwrap_or("127.0.0.1:8082".to_string());

    let tcp_addr = tcp_addr.parse::<SocketAddr>().unwrap();
    let udp_addr = udp_addr.parse::<SocketAddr>().unwrap();

    // Create the event loop that will drive this server
    let mut l = Core::new().unwrap();

    let tcp_handle = l.handle();
    let udp_handle = l.handle();

    let tcp_socket = TcpListener::bind(&tcp_addr, &tcp_handle).unwrap();
    let udp_socket = UdpSocket::bind(&udp_addr, &udp_handle).unwrap();

    println!("tcp listening on: {}", tcp_addr);
    println!("udp listening on: {}", udp_addr);

    let tcp_socket_handler = tcp_socket.incoming().for_each(|(socket, addr)| {
        let pair = futures::lazy(|| Ok(socket.split()));
        //copy data from reading to writing half
        let amt = pair.and_then(|(reader, writer)| copy(reader, writer));
        //spawn the future to allow it to run concurrently
        tcp_handle.spawn(amt.then(move |result| {
            println!("wrote {:?} bytes to {}", result, addr);
            Ok(())
        }));
        Ok(())
    });

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


    // Execute our server (modeled as a future) and wait for it to
    // complete.
    l.run(tcp_socket_handler).unwrap();
}
