#![feature(proc_macro)]

extern crate dns_parser;
extern crate toml;
extern crate futures;
extern crate chrono;
extern crate futures_cpupool;

extern crate serde;
//https://serde.rs/
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

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

use dns_parser::{Packet, QueryType};

mod types;
mod socket_read;
use socket_read::*;
mod socket_send;
use socket_send::*;

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

    //https://github.com/alexcrichton/futures-rs/blob/master/TUTORIAL.md#stream-example
    //https://tokio-rs.github.io/tokio-tls/tokio_tls/struct.ClientContext.html

    //TODO: readconfigfile if exists -> config, else -> defaultconfig
    let config = Arc::new(Config{
        addr: "www.rust-lang.org:443".to_socket_addrs().unwrap().next().unwrap(),
        pool: 4,
    });

    let pool = CpuPool::new(config.pool);
    let socket = UdpSocket::bind(&addr, &handle).unwrap();
    let requests = SocketReader::new(socket);

    let answer_attempts = requests.map(|(receiver_ref, buffer, amt)| {
        request_parser(config.clone(), receiver_ref.clone(), buffer, amt)
            .and_then(make_request)
    });

    let server = answer_attempts.for_each(|answer| {
        handle.spawn(pool.spawn(answer));
        Ok(())
    });

    core.run(server).unwrap();
}

fn request_parser(config: Arc<Config>, receiver: ReceiverRef, buffer: Buffer, amt: usize) -> BoxFuture<(ReceiverRef, Vec<Question>), ()> {
    log("parsing request");

    let mock_questions = vec![Question{
        qname: String::from("google.com"),
        qtype: 1,
    }];
    finished::<(ReceiverRef, Vec<Question>), ()>((receiver.clone(), mock_questions)).boxed();

    //TODO:
    //https://tailhook.github.io/dns-parser/dns_parser/struct.Packet.html
    if let Ok(packet) = Packet::parse(&buffer[..amt]) {
        log(&format!("packet parsed! ({},{},{})",
        packet.questions.len(),
        packet.answers.len(),
        packet.nameservers.len()));

        //only support question for now
        //https://groups.google.com/forum/#!topic/comp.protocols.dns.bind/uOWxNkm7AVg
        if packet.questions.len() != 1 {
            return failed::<(ReceiverRef, Vec<Question>), ()>(()).boxed()
        }

        let mut dns_questions = Vec::<Question>::new();
        for question in packet.questions {
            //only support those that the google API supports
            let qtype = match question.qtype {
                QueryType::A => 1,
                QueryType::AAAA => 28,
                QueryType::CNAME => 5,
                QueryType::MX => 15,
                QueryType::All => 255, //ANY
                _ => return failed::<(ReceiverRef, Vec<Question>), ()>(()).boxed()
            };
            dns_questions.push(Question{
                qname: question.qname.to_string(),
                qtype: qtype,
            });
        }
        //don't need to clone receiver when the mock is not in place
        //finished::<(ReceiverRef, Vec<Question>), ()>((receiver, dns_questions)).boxed()
        finished::<(ReceiverRef, Vec<Question>), ()>((receiver.clone(), dns_questions)).boxed()
    } else {
        failed::<(ReceiverRef, Vec<Question>), ()>(()).boxed()
    }
}

fn make_request((receiver, questions): (ReceiverRef, Vec<Question>)) -> BoxFuture<(), ()> {
    log("resolving request");

    //let buffer = [0; 1500];
    //let amt = 0;
    //SocketSender::new((receiver, buffer, amt))

    //spawning one thread (or process if its pthreads & epool)?
    //per tcp connection with its own eventloop could be correct..
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let addr = "dns.google.com:443".to_socket_addrs().unwrap().next().unwrap();
    //let addr = "www.rust-lang.org:443".to_socket_addrs().unwrap().next().unwrap();
    let stream = TcpStream::connect(&addr, &handle);

    let tls_handshake = stream.and_then(|socket| {
        let cx = ClientContext::new().unwrap();
        cx.handshake("dns.google.com", socket)
            //cx.handshake("www.rust-lang.org", socket)
    });

    //https://dns.google.com/resolve?name=www.rust-lang.org
    //TODO: https://dns.google.com/query?name=www.rust-lang.org&type=A&dnssec=true
    //this can also use the type as a number:
    //https://dns.google.com/query?name=www.rust-lang.org&type=1&dnssec=true

    //need to mock this, only temporary for testing if this works!
    //don't spam rust-lang.org!!
    let request = tls_handshake.and_then(|socket| {

        //let buffer = "\
        //GET / HTTP/1.0\r\n\
        //Host: www.rust-lang.org\r\n\
        //\r\n".as_bytes();
        let request = format!("\
                     GET /query?name={}&type={}&dnssec=true HTTP/1.1\r\n\
                     Host: dns.google.com\r\n\
                     \r\n", questions[0].qname, questions[0].qtype);
        let buffer = request.as_bytes().iter().cloned().collect::<Vec<u8>>();

        tokio_core::io::write_all(socket, buffer)
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
}
