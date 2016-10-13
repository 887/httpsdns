extern crate toml;
extern crate futures;
extern crate tokio_core;
extern crate dns_parser;

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
    // Execute our server (modeled as a future) and wait for it to
    // complete.
    l.run(tcp_socket_handler).unwrap();

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

    //oh.. there is an example that does what i need?
    //https://github.com/alexcrichton/futures-trustdns-test/blob/5ebd74bfa041923bd2b44b14cc818f5511b80767/src/main.rs

    //more resources: http://stackoverflow.com/questions/39049365/rust-echo-server-and-client-using-futures-blocks-itself-forever
    //(build a client to test it?)
}
