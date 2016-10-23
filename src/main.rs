#![feature(proc_macro)]

extern crate dns_parser;
extern crate toml; //TODO: configuration file
extern crate futures;
extern crate chrono;
extern crate futures_cpupool;
extern crate serde; //https://serde.rs
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate tokio_tls; //https://github.com/tokio-rs/tokio-tls/blob/master/Cargo.toml
#[macro_use]
extern crate tokio_core;
extern crate http_muncher;

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

use http_muncher::{Parser, ParserHandler};

mod types;
mod socket_read;
use socket_read::*;
mod socket_send;
use socket_send::*;

use types::*;

// test udp port https://wiki.itadmins.net/network/tcp_udp_ping
// sudo watch -n 5 "nmap -P0 -sU -p54321 127.0.0.1"
// run with:
// cargo run --features "mock_request"

fn main() {
    let addr = env::args().nth(1).unwrap_or("127.0.0.1:54321".to_string());
    let addr = addr.parse::<SocketAddr>().unwrap();

    let mut core = Core::new().unwrap();
    let handle = core.handle();

    // TODO: read configuration file if exists -> config, else -> defaultconfig
    let config = Arc::new(Config {
        https_dns_server_name: "dns.google.com".to_string(),
        https_dns_server_port: 443,
        https_dns_server_addr: "dns.google.com:443".to_socket_addrs().unwrap().next().unwrap(),
        pool: 4,
    });

    let pool = CpuPool::new(config.pool);
    let socket = UdpSocket::bind(&addr, &handle).unwrap();
    let requests = SocketReader::new(socket);

    let answer_attempts = requests.map(|(receiver_ref, buffer, amt)| {
        request_parser(config.clone(), receiver_ref.clone(), buffer, amt).and_then(make_request)
    });

    let server = answer_attempts.for_each(|answer| {
        handle.spawn(pool.spawn(answer));
        Ok(())
    });

    core.run(server).unwrap();
}

#[cfg(feature = "mock_request")]
fn request_parser(config: Arc<Config>,
                  receiver: ReceiverRef,
                  buffer: Buffer,
                  amt: usize)
                  -> BoxFuture<ParsedRequest, ()> {
    log("request mocked");
    let mock_questions = vec![Question {
                                  qname: String::from("google.com"),
                                  qtype: 1,
                              }];
    finished::<ParsedRequest, ()>((config, receiver, mock_questions)).boxed()
}

#[cfg(not(feature = "mock_request"))]
fn request_parser(config: Arc<Config>,
                  receiver: ReceiverRef,
                  buffer: Buffer,
                  amt: usize)
                  -> BoxFuture<ParsedRequest, ()> {
    log("parsing request");
    // https://tailhook.github.io/dns-parser/dns_parser/struct.Packet.html
    if let Ok(packet) = Packet::parse(&buffer[..amt]) {
        log(&format!("packet parsed! ({},{},{})",
                     packet.questions.len(),
                     packet.answers.len(),
                     packet.nameservers.len()));

        // only support one question
        // https://groups.google.com/forum/#!topic/comp.protocols.dns.bind/uOWxNkm7AVg
        if packet.questions.len() != 1 {
            return failed::<ParsedRequest, ()>(()).boxed();
        }

        let mut dns_questions = Vec::<Question>::new();
        for question in packet.questions {
            // only support those that the google API supports
            let qtype = match question.qtype {
                QueryType::A => 1,
                QueryType::AAAA => 28,
                QueryType::CNAME => 5,
                QueryType::MX => 15,
                QueryType::All => 255, //ANY
                _ => return failed::<ParsedRequest, ()>(()).boxed(),
            };
            dns_questions.push(Question {
                qname: question.qname.to_string(),
                qtype: qtype,
            });
        }
        finished::<ParsedRequest, ()>((config, receiver, dns_questions)).boxed()
    } else {
        failed::<ParsedRequest, ()>(()).boxed()
    }
}

#[cfg(feature = "mock_answer")]
fn make_request((config, receiver, questions): ParsedRequest) -> BoxFuture<(), ()> {
    log("answer mocked");
    let buffer = [0; 1500];
    let amt = 0;
    SocketSender::new((receiver, buffer, amt)).boxed()
}

#[cfg(not(feature = "mock_answer"))]
fn make_request((config, receiver, questions): ParsedRequest) -> BoxFuture<(), ()> {
    log("resolving answer");

    // https://github.com/alexcrichton/futures-rs/blob/master/TUTORIAL.md#stream-example
    // https://tokio-rs.github.io/tokio-tls/tokio_tls/struct.ClientContext.html

    // eventloop in eventloop?
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let stream = TcpStream::connect(&config.https_dns_server_addr, &handle);

    let tls_handshake = stream.and_then(|socket| {
        let cx = ClientContext::new().unwrap();
        cx.handshake(&config.https_dns_server_name, socket)
    });

    let request = tls_handshake.and_then(|socket| {

        // https://developers.google.com/speed/public-dns/docs/dns-over-https

        // machine API:
        // https://dns.google.com/resolve?name=www.rust-lang.org
        // human API:
        // https://dns.google.com/query?name=www.rust-lang.org&type=A&dnssec=true
        // this can also use the type as a number:
        // https://dns.google.com/query?name=www.rust-lang.org&type=1&dnssec=true
        let request = format!("GET /resolve?name={}&type={}&dnssec=true HTTP/1.0\r\n\
                               Host: dns.google.com\r\n\r\n",
                              questions[0].qname,
                              questions[0].qtype);
        let buffer = request.as_bytes().iter().cloned().collect::<Vec<u8>>();

        tokio_core::io::write_all(socket, buffer)
    });
    let response = request.and_then(|(socket, _)| {
        tokio_core::io::read_to_end(socket, Vec::new()).boxed()
    });
    match core.run(response) {
        Ok((_, data)) => {
            log(&format!("{} bytes read!", data.len()));

            //there are still HTTP headers in here, need to strip those and just access the body
            //let answer_str = String::from_utf8_lossy(&data);
            //log(&answer_str);

            struct BodyHandler(String);
            impl ParserHandler for BodyHandler{
                fn on_body(&mut self, _: &mut Parser, body: &[u8]) -> bool {
                    self.0 = String::from_utf8_lossy(body).to_string();
                    true
                }
            }
            let mut body_handler = BodyHandler(String::new());

            let mut parser = Parser::response();
            parser.parse(&mut body_handler, &data);

            let body = body_handler.0;
            //log(&format!("body: {}",body));

            if let Ok(deserialized) = serde_json::from_str::<Request>(&body) {
                println!("deserialized = {:?}", deserialized);
                //TODO build a dns_parser Packet out of it and send it back!
                //(PS: this bufffer is probably not long enough)
                let mut arr = [0u8; 1500];
                let mut len = 1500;
                if data.len() < 1500 {
                    len = data.len();
                }
                for i in 0..len {
                    arr[i] = data[i];
                }
                SocketSender::new((receiver, arr, len)).boxed()
            } else {
                finished::<(), ()>(()).boxed()
            }
        }
        Err(_) => finished::<(), ()>(()).boxed(),
    }
}
