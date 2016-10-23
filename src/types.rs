use chrono::Local;

use std::net::SocketAddr;
use tokio_core::net::UdpSocket;

// use std::rc::Rc;
use std::sync::Arc;

pub struct Receiver {
    pub socket: Arc<UdpSocket>,
    pub addr: SocketAddr,
}
pub type ReceiverRef = Arc<(Receiver)>;

pub struct Config {
    pub https_dns_server_name: String,
    pub https_dns_server_port: u16,
    pub https_dns_server_addr: SocketAddr,
    pub pool: usize,
}

pub type ParsedRequest = (Arc<Config>, ReceiverRef, Vec<Question>);

pub type Buffer = [u8; 1500];

pub fn log(text: &str) {
    println!("{}: {}",
             Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
             text);
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Question {
    #[serde(rename="name")]
    pub qname: String,
    #[serde(rename="type")]
    pub qtype: u16,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Answer {
    #[serde(rename="name")]
    pub aname: String,
    #[serde(rename="type")]
    pub atype: u16,
    #[serde(rename="TTL")]
    pub ttl: u32,
    pub data: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Authority {
    #[serde(rename="name")]
    pub aname: String,
    #[serde(rename="type")]
    pub atype: u16,
    #[serde(rename="TTL")]
    pub ttl: u32,
    pub data: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
    #[serde(rename="Status")]
    pub status: u32,
    #[serde(rename="TC")]
    pub tc: bool,
    #[serde(rename="RD")]
    pub rd: bool,
    #[serde(rename="RA")]
    pub ra: bool,
    #[serde(rename="AD")]
    pub ad: bool,
    #[serde(rename="CD")]
    pub cd: bool,
    #[serde(rename="Question")]
    pub question: Vec<Question>,
    #[serde(rename="Answer")]
    pub answer: Option<Vec<Answer>>,
    #[serde(rename="Comment")]
    pub comment: Option<String>,
}
