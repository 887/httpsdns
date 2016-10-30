#![feature(proc_macro)]
#![feature(test)]

extern crate dns_parser;
extern crate toml;
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
extern crate test;
#[macro_use]
extern crate cfg_if;

use std::env;
use std::net::{SocketAddr, ToSocketAddrs};

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

mod types;
mod socket_read;
use socket_read::*;
mod socket_send;
use socket_send::*;

use types::*;

//static CERT: &'static str = "-----BEGIN CERTIFICATE-----
static CERT: &'static [u8] = b"-----BEGIN CERTIFICATE-----
MIIHEjCCBfqgAwIBAgIIfqB5Y6IWFkkwDQYJKoZIhvcNAQELBQAwSTELMAkGA1UE
BhMCVVMxEzARBgNVBAoTCkdvb2dsZSBJbmMxJTAjBgNVBAMTHEdvb2dsZSBJbnRl
cm5ldCBBdXRob3JpdHkgRzIwHhcNMTYxMDE5MTczNjEzWhcNMTcwMTExMTcxMzAw
WjBmMQswCQYDVQQGEwJVUzETMBEGA1UECAwKQ2FsaWZvcm5pYTEWMBQGA1UEBwwN
TW91bnRhaW4gVmlldzETMBEGA1UECgwKR29vZ2xlIEluYzEVMBMGA1UEAwwMKi5n
b29nbGUuY29tMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEZe1k6paKfkHsCDRK
0qy8r+tdvK8PTLJwfouLgERIZeDG12Iwx35KPkKq24/CmXMeHBmmXty9x3hmqioz
3sDRaKOCBKowggSmMB0GA1UdJQQWMBQGCCsGAQUFBwMBBggrBgEFBQcDAjCCA2kG
A1UdEQSCA2AwggNcggwqLmdvb2dsZS5jb22CDSouYW5kcm9pZC5jb22CFiouYXBw
ZW5naW5lLmdvb2dsZS5jb22CEiouY2xvdWQuZ29vZ2xlLmNvbYIWKi5nb29nbGUt
YW5hbHl0aWNzLmNvbYILKi5nb29nbGUuY2GCCyouZ29vZ2xlLmNsgg4qLmdvb2ds
ZS5jby5pboIOKi5nb29nbGUuY28uanCCDiouZ29vZ2xlLmNvLnVrgg8qLmdvb2ds
ZS5jb20uYXKCDyouZ29vZ2xlLmNvbS5hdYIPKi5nb29nbGUuY29tLmJygg8qLmdv
b2dsZS5jb20uY2+CDyouZ29vZ2xlLmNvbS5teIIPKi5nb29nbGUuY29tLnRygg8q
Lmdvb2dsZS5jb20udm6CCyouZ29vZ2xlLmRlggsqLmdvb2dsZS5lc4ILKi5nb29n
bGUuZnKCCyouZ29vZ2xlLmh1ggsqLmdvb2dsZS5pdIILKi5nb29nbGUubmyCCyou
Z29vZ2xlLnBsggsqLmdvb2dsZS5wdIISKi5nb29nbGVhZGFwaXMuY29tgg8qLmdv
b2dsZWFwaXMuY26CFCouZ29vZ2xlY29tbWVyY2UuY29tghEqLmdvb2dsZXZpZGVv
LmNvbYIMKi5nc3RhdGljLmNugg0qLmdzdGF0aWMuY29tggoqLmd2dDEuY29tggoq
Lmd2dDIuY29tghQqLm1ldHJpYy5nc3RhdGljLmNvbYIMKi51cmNoaW4uY29tghAq
LnVybC5nb29nbGUuY29tghYqLnlvdXR1YmUtbm9jb29raWUuY29tgg0qLnlvdXR1
YmUuY29tghYqLnlvdXR1YmVlZHVjYXRpb24uY29tggsqLnl0aW1nLmNvbYIaYW5k
cm9pZC5jbGllbnRzLmdvb2dsZS5jb22CC2FuZHJvaWQuY29tggRnLmNvggZnb28u
Z2yCFGdvb2dsZS1hbmFseXRpY3MuY29tggpnb29nbGUuY29tghJnb29nbGVjb21t
ZXJjZS5jb22CGXBvbGljeS5tdGEtc3RzLmdvb2dsZS5jb22CCnVyY2hpbi5jb22C
Cnd3dy5nb28uZ2yCCHlvdXR1LmJlggt5b3V0dWJlLmNvbYIUeW91dHViZWVkdWNh
dGlvbi5jb20wCwYDVR0PBAQDAgeAMGgGCCsGAQUFBwEBBFwwWjArBggrBgEFBQcw
AoYfaHR0cDovL3BraS5nb29nbGUuY29tL0dJQUcyLmNydDArBggrBgEFBQcwAYYf
aHR0cDovL2NsaWVudHMxLmdvb2dsZS5jb20vb2NzcDAdBgNVHQ4EFgQUBHPtSLlP
2Lw8BjYDEjn+KuNuB5wwDAYDVR0TAQH/BAIwADAfBgNVHSMEGDAWgBRK3QYWG7z2
aLV29YG2u2IaulqBLzAhBgNVHSAEGjAYMAwGCisGAQQB1nkCBQEwCAYGZ4EMAQIC
MDAGA1UdHwQpMCcwJaAjoCGGH2h0dHA6Ly9wa2kuZ29vZ2xlLmNvbS9HSUFHMi5j
cmwwDQYJKoZIhvcNAQELBQADggEBAGWQo/gApR9Ggt/4avmMwtEu5J29UYMzxw4i
WwDF/cKd4gPnPQxFb6zBhCYyJwNR+Z3XTK2Ldjexfb1IEPEif5RBfJFwwr1jCFDp
srsDuv3EugyeXGfOv3u9zwZg2zNwNJswjNRDd47P8voISSgo0hBy7DcRKyvX8UYD
ybVYzoGeRzLW8awDWZaqRkX8IglWw4IB63WrtevrUg0gYjnErYYlbrBgQFdjmhTZ
nZwLGga27MCyDnfLRwiQBZ+D6JIHPdcqxekQuzNBynqrOpT2FEOVJ8N+BKORP18v
PA351/jGssrZKdRYOpI2KwMOm+c1z8yReeSD6G55pANNeQhAco8=
-----END CERTIFICATE-----";

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

fn main_proxy() {
    let addr = env::args().nth(1).unwrap_or("0.0.0.0:54321".to_string());
    log(&format!("listening on: {}", addr));
    let addr = addr.parse::<SocketAddr>().unwrap();

    let mut core = Core::new().unwrap();
    let handle = core.handle();

    //google ips:
    //4.31.115.251

    // TODO: read configuration file if exists -> config, else -> defaultconfig
    let config = Arc::new(Config {
        //name during the hanshake & GET request (may be split into two parameters later)
        https_dns_server_name: "dns.google.com".to_string(),
        //ip of the server we connect to (this will also be resolved if its an adress,
        //but then you can't replace the system DNS server)
        https_dns_server_addr: "4.31.115.251:443".to_socket_addrs().unwrap().next().unwrap(),
        //https_dns_server_addr: "dns.google.com:443".to_socket_addrs().unwrap().next().unwrap(),
        pool: 4,
    });

    let pool = CpuPool::new(config.pool);
    let socket = UdpSocket::bind(&addr, &handle).unwrap();
    let requests = SocketReader::new(socket);

    let answer_attempts = requests.map(|(receiver_ref, buffer, amt)| {
        handle_request(config.clone(), receiver_ref.clone(), buffer, amt)
    });

    let server = answer_attempts.for_each(|answer| {
        handle.spawn(pool.spawn(answer));
        Ok(())
    });

    core.run(server).unwrap();
}

fn handle_request(config: Arc<Config>,
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
            handle_packet(config, receiver, packet)
        }
    } else {
        finished::<(), ()>(()).boxed()
    }
}


fn handle_packet(config: Arc<Config>, receiver: ReceiverRef, packet: Packet) -> BoxFuture<(), ()> {
    log("resolving answer");

    // https://github.com/alexcrichton/futures-rs/blob/master/TUTORIAL.md#stream-example
    // https://tokio-rs.github.io/tokio-tls/tokio_tls/struct.ClientContext.html

    // eventloop in eventloop?
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let stream = TcpStream::connect(&config.https_dns_server_addr, &handle);

    let tls_handshake = stream.and_then(|socket| {
        let mut cx = ClientContext::new().unwrap();
        //TODO import exensions like shown here to have this function
        //https://github.com/tokio-rs/tokio-tls/blob/master/src/lib.rs
        {
            let ssqlcontext = cx.ssl_context_mut();
                if cfg!(feature = "rustls") {
                } else if cfg!(any(feature = "force-openssl",
                            all(not(target_os = "macos"),
                            not(target_os = "windows")))) {
                    //https://sfackler.github.io/rust-openssl/doc/v0.8.3/openssl/ssl/struct.SslContext.html

                    use ossl::x509::*;
                    let cert = X509::from_pem(CERT).unwrap();
                    let cert_ref = unsafe {X509Ref::from_ptr(cert.as_ptr())};

                    ssqlcontext.set_certificate(&cert_ref).ok();
                } else if cfg!(target_os = "macos") {
                } else {
                }
        }
        cx.handshake(&config.https_dns_server_name, socket)
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
                              &config.https_dns_server_name);
        let buffer = request.as_bytes().iter().cloned().collect::<Vec<u8>>();
        tokio_core::io::write_all(socket, buffer)
    });
    let response = request.and_then(|(socket, _)| {
        tokio_core::io::read_to_end(socket, Vec::new()).boxed()
    });
    if let Ok((_, data)) = core.run(response) {
        log(&format!("{} bytes read!", data.len()));
        deserialize_answer(&config, receiver, packet, data)
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
                      packet: Packet,
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
                  packet: Packet,
                  deserialized: Request)
                  -> BoxFuture<(), ()> {

    // apparently this part was already done:
    // https://github.com/gmosley/rust-DNSoverHTTPS
    // https://david-cao.github.io/rustdocs/dns_parser/

    // the only reason to keep the incoming packet around is this id, maybe drop the rest?
    let mut response = Builder::new_response(packet.header.id,
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

