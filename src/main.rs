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

use dns_parser::{Packet, QueryType, Builder, Type, QueryClass, Class, ResponseCode};

use http_muncher::{Parser, ParserHandler};

mod types;
mod socket_read;
use socket_read::*;
mod socket_send;
use socket_send::*;

use types::*;

// test udp port https://wiki.itadmins.net/network/tcp_udp_ping
// sudo watch -n 5 "nmap -P0 -sU -p54321 127.0.0.1"
// this creates a zero-len udp package, that we can use to mock a request

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
        make_request(config.clone(), receiver_ref.clone(), buffer, amt)
    });

    let server = answer_attempts.for_each(|answer| {
        handle.spawn(pool.spawn(answer));
        Ok(())
    });

    core.run(server).unwrap();
}

#[cfg(feature = "mock_answer")]
fn make_request(_: Arc<Config>,
                receiver: ReceiverRef,
                buffer: Buffer,
                amt: usize) -> BoxFuture<(), ()> {
    log("answer mocked");
    let buffer = buffer[..amt].iter().cloned().collect::<Vec<u8>>();
    SocketSender::new((receiver, buffer)).boxed()
}

#[cfg(not(feature = "mock_answer"))]
fn make_request(config: Arc<Config>,
                 receiver: ReceiverRef,
                 mut buffer: Buffer,
                 mut amt: usize) -> BoxFuture<(), ()> {
    log("resolving answer");

    if amt == 0 {
        log("mocking request");
        let mut b = Builder::new_query(0,false);
        b.add_question("google.com", QueryType::A, QueryClass::Any);
        let data = match b.build() {
            Ok(data) | Err(data) => data,
        };
        amt = if data.len() < 1500 { data.len() } else { 1500 };
        for i in 0..amt {
            buffer[i] = data[i];
        }
    }

    // https://tailhook.github.io/dns-parser/dns_parser/struct.Packet.html
    if let Ok(packet) = Packet::parse(&buffer[..amt]) {
        // only support one question
        // https://groups.google.com/forum/#!topic/comp.protocols.dns.bind/uOWxNkm7AVg
        if packet.questions.len() != 1 {
            log(&format!("packet questions != 1 (amt: {})", packet.questions.len()));
            finished::<(), ()>(()).boxed()
        } else {
            log(&format!("packet parsed!"));
            handle_packet(config,receiver,packet)
        }
    } else {
        finished::<(), ()>(()).boxed()
    }
}


fn handle_packet(config: Arc<Config>,
                receiver: ReceiverRef,
                packet: Packet) -> BoxFuture<(), ()> {
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

    let qtype = packet.questions[0].qtype as u16;
    let qname = packet.questions[0].qname.to_string();
    log(&format!("requesting name:{}, type{}", qname, qtype));

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
                                   qname,
                                   qtype);
        let buffer = request.as_bytes().iter().cloned().collect::<Vec<u8>>();

        tokio_core::io::write_all(socket, buffer)
    }).and_then(|(socket, _)| {
        tokio_core::io::read_to_end(socket, Vec::new()).boxed()
    });
    if let Ok((_, data)) = core.run(request) {
        log(&format!("{} bytes read!", data.len()));
        parse_answer(&config, receiver, packet, data)
    } else {
        finished::<(), ()>(()).boxed()
    }
}

fn parse_answer(config: &Arc<Config>,
                receiver: ReceiverRef,
                packet: Packet,
                data: Vec<u8>) -> BoxFuture<(), ()> {

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

        //apparently this part was allready done https://github.com/gmosley/rust-DNSoverHTTPS
        //there is a forked verison of dns_parser that supports what i need here:
        //https://david-cao.github.io/rustdocs/dns_parser/

        let mut response = Builder::new_response(
            //the only reason to keep the packet arround is this id, might consider
            //moving it?
            packet.header.id,
            ResponseCode::NoError,
            deserialized.tc,
            deserialized.rd,
            deserialized.ra
            );

        for question in deserialized.questions {
            let query_type = QueryType::parse(question.qtype).unwrap();
            response.add_question(
                &remove_fqdn_dot(&question.qname),
                query_type,
                QueryClass::IN
                );
        }

        if let Some(answers) = deserialized.answers {
            for answer in answers {
                if let Ok(data) = answer.write() {
                    response.add_answer(
                        &remove_fqdn_dot(&answer.aname),
                        Type::parse(answer.atype).unwrap(),
                        Class::IN,
                        answer.ttl,
                        data,
                        );
                }
            }
        }

        let data = match response.build() {
            Ok(data) | Err(data) => data,
        };

        //todo: improve this buffer!
        //(PS: this bufffer is probably not long enough)
        //let mut arr = [0u8; 1500];
        //let len = if data.len() < 1500 { data.len() } else { 1500 };
        //for i in 0..len {
        //arr[i] = data[i];
        //}
        SocketSender::new((receiver, data)).boxed()
    } else {
        finished::<(), ()>(()).boxed()
    }
}

fn remove_fqdn_dot(domain_name: &str) -> String {
    let mut domain_name_string = domain_name.to_owned();
    domain_name_string.pop();
    domain_name_string
}
