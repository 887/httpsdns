#![feature(proc_macro)]
#![feature(test)]

extern crate dns_parser;
extern crate futures;
extern crate chrono;
extern crate futures_cpupool;
extern crate serde; //https://serde.rs
//#[macro_use]
//extern crate serde_derive; //todo: hybrid approach? https://serde.rs/codegen-hybrid.html
extern crate serde_json;
extern crate tokio_tls; //https://github.com/tokio-rs/tokio-tls/blob/master/Cargo.toml
#[macro_use]
extern crate tokio_core;
extern crate http_muncher;
extern crate test;
#[macro_use]
extern crate cfg_if;
extern crate toml;
extern crate env_logger;

include!(concat!(env!("OUT_DIR"), "/serde_types.rs"));

use std::env;
use std::net::{SocketAddr};

use std::sync::Arc;

use tokio_core::net::{TcpStream,UdpSocket};
use tokio_core::reactor::Core;

use futures::*;
use futures::stream::Stream;
use futures_cpupool::CpuPool;

#[cfg(feature = "server")]
use tokio_core::net::TcpListener;

use tokio_tls::{ClientContext};
cfg_if! {
    if #[cfg(feature = "rustls")] {
        use tokio_tls::backend::rustls;
        use tokio_tls::backend::rustls::ClientContextExt;
    } else if #[cfg(any(feature = "force-openssl",
              all(not(target_os = "macos"), not(target_os = "windows"))))] {
        extern crate openssl as ossl;
        use tokio_tls::backend::openssl::ClientContextExt;
    } else if #[cfg(target_os = "macos")] {
        use tokio_tls::backend::secure_transport;
        use tokio_tls::backend::secure_transport::ClientContextExt;
    } else {
        use tokio_tls::backend::schannel;
        use tokio_tls::backend::schannel::ClientContextExt;
    }
}
use dns_parser::{Packet, QueryType, Builder, Type, QueryClass, Class, ResponseCode};
use http_muncher::{Parser, ParserHandler};
use std::fs::{File};
use std::path::Path;
use std::io::{Read};
use toml::Value;

mod types;
mod socket_read;
use socket_read::*;
mod socket_send;
use socket_send::*;

use types::*;

// test udp port https://wiki.itadmins.net/network/tcp_udp_ping
// sudo watch -n 5 "nmap -P0 -sU -p54321 127.0.0.1"
// this creates a zero-len udp package, that is used to mock a request

// also testable as real dns proxy on linux:
// cargo build
// sudo RUST_BACKTRACE=1 ./target/debug/httpsdns 0.0.0.0:53
// put a new line with "nameserver 127.0.0.1" in /etc/resolf.conf
// (obviously: comment out the old with a # or backup it otherwise)

#[cfg(feature = "server")]
fn main() { main_server() }

#[cfg(not(feature = "server"))]
fn main() { main_proxy() }

fn read_config() -> Arc<Config> {
    // TODO override with config from cmdlline or take it from /etc
    let config_path = Path::new("Config.toml");
    let mut input_text = String::new();
    File::open(config_path).unwrap().read_to_string(&mut input_text).unwrap();

    println!("{}", input_text);

    let mut parser = toml::Parser::new(&input_text);
    let toml = match parser.parse() {
        Some(value) => {
            println!("found toml: {:?}", value);
            Some(value)
        },
        None => {
            println!("parse errors: {:?}", parser.errors);
            None
        }
    };
    let config = Value::Table(toml.unwrap());

    let config_toml: ConfigToml = toml::decode(config).unwrap();
    let config: Arc<Config> = Arc::new(config_toml.config);
    println!("{}", config.api_server_name);

    config
}

fn read_cert_file(path: &Path) -> Vec<u8> {
    let mut buf = Vec::<u8>::new();
    File::open(path).unwrap().read_to_end(&mut buf).ok();
    buf
}

fn main_proxy() {
    //TODO make this an option of the config.toml
    let addr = env::args().nth(1).unwrap_or("0.0.0.0:54321".to_string());
    log(&format!("listening on: {}", addr));
    let addr = addr.parse::<SocketAddr>().unwrap();

    let config = read_config();
    let cert_path = Path::new(&config.api_cert_path);
    let cert = read_cert_file(cert_path);
    let cert_ref = Arc::new(cert);

    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let pool = CpuPool::new(config.cpu_pool as usize);
    let socket = UdpSocket::bind(&addr, &handle).unwrap();
    let requests = SocketReader::new(socket);

    let answer_attempts = requests.map(|(receiver_ref, buffer, amt)| {
        handle_request(config.clone(), cert_ref.clone(), receiver_ref.clone(), buffer, amt)
    });

    let server = answer_attempts.for_each(|answer| {
        handle.spawn(pool.spawn(answer));
        Ok(())
    });

    core.run(server).unwrap();
}

fn handle_request(config: Arc<Config>,
                  cert: Arc<Vec<u8>>,
                  receiver: ReceiverRef,
                  mut buffer: Buffer,
                  mut amt: usize)
                  -> BoxFuture<(), ()> {
    log(&format!("resolving answer {}", amt));

    // test this with:
    // sudo watch -n 5 "nmap -P0 -sU -p54321 127.0.0.1"
    if amt <= 12 {
        log("mocking request");
        let mut b = Builder::new_query(0, false);
        b.add_question("google.com", QueryType::A, QueryClass::Any);
        let data = match b.build() {
            Ok(data) | Err(data) => data,
        };
        amt = if data.len() < 1500 {
            data.len()
        } else {
            1500
        };
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
            log("packet parsed!");
            handle_packet(config, cert, receiver, packet)
        }
    } else {
        finished::<(), ()>(()).boxed()
    }
}

fn handle_packet(config: Arc<Config>,
                 cert: Arc<Vec<u8>>,
                 receiver: ReceiverRef,
                 packet: Packet) -> BoxFuture<(), ()> {
    log("resolving answer");

    // https://github.com/alexcrichton/futures-rs/blob/master/TUTORIAL.md#stream-example
    // https://tokio-rs.github.io/tokio-tls/tokio_tls/struct.ClientContext.html

    // eventloop in eventloop?
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let stream = TcpStream::connect(&config.api_server_addr, &handle);

    let tls_handshake = stream.and_then(|socket| {
        let mut cx = ClientContext::new().unwrap();
        {
            let ssqlcontext = cx.ssl_context_mut();
                if cfg!(feature = "rustls") {
                } else if cfg!(any(feature = "force-openssl",
                            all(not(target_os = "macos"),
                            not(target_os = "windows")))) {


                    //https://sfackler.github.io/rust-openssl/doc/v0.8.3/openssl/ssl/struct.SslContext.html
                    use ossl::x509::*;
                    let cert = X509::from_pem(&cert).unwrap();
                    let cert_ref = unsafe {X509Ref::from_ptr(cert.as_ptr())};

                    ssqlcontext.set_certificate(&cert_ref).ok();
                } else if cfg!(target_os = "macos") {
                } else {
                }
        }
        cx.handshake(&config.api_server_name, socket)
    });

    let qtype = packet.questions[0].qtype as u16;
    let qname = packet.questions[0].qname.to_string();
    log(&format!("requesting name:{}, type:{}", qname, qtype));

    let request = tls_handshake.and_then(|socket| {
        // https://developers.google.com/speed/public-dns/docs/dns-over-https
        let request = format!("GET /resolve?name={}&type={}&dnssec=true HTTP/1.0\r\nHost: \
                               {}\r\n\r\n",
                              qname,
                              qtype,
                              &config.api_server_name);
        let buffer = request.as_bytes().iter().cloned().collect::<Vec<u8>>();
        tokio_core::io::write_all(socket, buffer)
    });
    let response = request.and_then(|(socket, _)| {
        tokio_core::io::read_to_end(socket, Vec::new()).boxed()
    });
    if let Ok((_, data)) = core.run(response) {
        log(&format!("{} bytes read!", data.len()));
        deserialize_answer(&config, receiver, ParsedPacket{id: packet.header.id}, data)
    } else {
        finished::<(), ()>(()).boxed()
    }
}

struct BodyHandler(String);
impl ParserHandler for BodyHandler {
    fn on_body(&mut self, _: &mut Parser, body: &[u8]) -> bool {
        self.0 = String::from_utf8_lossy(body).to_string();
        true
    }
}

fn deserialize_answer(config: &Arc<Config>,
                      receiver: ReceiverRef,
                      packet: ParsedPacket,
                      data: Vec<u8>)
                      -> BoxFuture<(), ()> {

    let mut body_handler = BodyHandler(String::new());
    let mut parser = Parser::response();
    parser.parse(&mut body_handler, &data);
    let body = body_handler.0;
    if let Ok(deserialized) = serde_json::from_str::<Request>(&body) {
        println!("deserialized = {:?}", deserialized);
        build_response(config, receiver, packet, deserialized)
    } else {
        finished::<(), ()>(()).boxed()
    }
}

fn build_response(_: &Arc<Config>,
                  receiver: ReceiverRef,
                  packet: ParsedPacket,
                  deserialized: Request)
                  -> BoxFuture<(), ()> {

    // apparently this part was already done:
    // https://github.com/gmosley/rust-DNSoverHTTPS
    // https://david-cao.github.io/rustdocs/dns_parser/

    // the only reason to keep the incoming packet around is this id, maybe drop the rest?
    let mut response = Builder::new_response(packet.id,
                                             ResponseCode::NoError,
                                             deserialized.tc,
                                             deserialized.rd,
                                             deserialized.ra);

    for question in deserialized.questions {
        let query_type = QueryType::parse(question.qtype).unwrap();
        response.add_question(&remove_fqdn_dot(&question.qname),
                              query_type,
                              QueryClass::IN);
    }

    if let Some(answers) = deserialized.answers {
        for answer in answers {
            if let Ok(data) = answer.write() {
                response.add_answer(&remove_fqdn_dot(&answer.aname),
                                    Type::parse(answer.atype).unwrap(),
                                    Class::IN,
                                    answer.ttl,
                                    data);
            }
        }
    }

    let data = match response.build() {
        Ok(data) | Err(data) => data,
    };

    SocketSender::new((receiver, data)).boxed()
}

/// Workarround: dns-parser improperly formats
fn remove_fqdn_dot(domain_name: &str) -> String {
    let mut domain_name_string = domain_name.to_owned();
    domain_name_string.pop();
    domain_name_string
}

#[cfg(feature = "server")]
fn main_server() {
    //no tls
    let mut core = Core::new().unwrap();
    let address = "127.0.0.1:8080".parse().unwrap();
    let listener = TcpListener::bind(&address, &core.handle()).unwrap();

    let addr = listener.local_addr().unwrap();
    println!("Listening for connections on {}", addr);

    let clients = listener.incoming();
    let welcomes = clients.and_then(|(socket, _peer_addr)| {
        tokio_core::io::read_to_end(socket, Vec::new())
    });
    let response = welcomes.map(|(socket, data)| {
        let mut body_handler = BodyHandler(String::new());
        let mut parser = Parser::response();
        parser.parse(&mut body_handler, &data);
        let body = body_handler.0;
        if let Ok(deserialized) = serde_json::from_str::<Request>(&body) {
            println!("deserialized = {:?}", deserialized);
            tokio_core::io::write_all(socket, b"Hello!\n");
        }
        finished::<(), ()>(()).boxed()
    });
    //let server = response.for_each(|(_socket, _welcome)| {
    let server = response.for_each(|_| {
        Ok(())
    });

    core.run(server).unwrap();
}

//BIG TODO: benchtest all of this an make the code testable (
pub fn add_two(a: i32) -> i32 {
    a + 2
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    fn one_eventloop_100(b: &mut Bencher) {
        b.iter(|| {
            main_server();
            main_proxy();
        });
    }
    #[bench]
    fn two_eventloops_100(b: &mut Bencher) {
        b.iter(|| {

        });
    }
    #[bench]
    fn two_eventloops_cpupool_100(b: &mut Bencher) {
        b.iter(|| {

        });
    }
}

impl Answer {
    pub fn write(&self) -> Result<Vec<u8>, ()> {
        use std::net::{Ipv4Addr, Ipv6Addr};
        use std::str::FromStr;

        match self.atype {
            1 => {
                let ip = Ipv4Addr::from_str(&self.data).unwrap();
                Ok(ip.octets().to_vec())
            }
            5 | 12 => {
                let mut data: Vec<u8> = Vec::new();
                let name = &self.data;
                for label in name.split('.') {
                    let size = label.len() as u8;
                    data.push(size);
                    data.extend(label.as_bytes());
                }
                Ok(data)
            }
            28 => {
                let ip = Ipv6Addr::from_str(&self.data).unwrap();
                let mut ipv6_bytes: Vec<u8> = Vec::new();
                for segment in &ip.segments() {
                    let upper = segment >> 8;
                    let lower = segment & 0b0000_0000_1111_1111;
                    ipv6_bytes.push(upper as u8);
                    ipv6_bytes.push(lower as u8);
                }
                Ok(ipv6_bytes)
            }
            _ => Err(()),
        }
    }
}

