use chrono::Local;

use std::net::SocketAddr;
use tokio_core::net::UdpSocket;

// use std::rc::Rc;
use std::sync::Arc;

pub struct Receiver {
    pub socket: Arc<UdpSocket>,
    pub addr: SocketAddr
}
pub type ReceiverRef = Arc<(Receiver)>;

pub struct Config{
    pub addr: SocketAddr,
    pub pool: usize,
}

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
    pub qtype: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Answer {
    #[serde(rename="name")]
    pub aname: String,
    #[serde(rename="type")]
    pub atype: u32,
    #[serde(rename="TTL")]
    pub ttl: u32,
    pub data: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Authority {
    #[serde(rename="name")]
    pub aname: String,
    #[serde(rename="type")]
    pub atype: u32,
    #[serde(rename="TTL")]
    pub ttl: u32,
    pub data: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Request {
    #[serde(rename="Status")]
    status: u32,
    #[serde(rename="TC")]
    tc: bool,
    #[serde(rename="RD")]
    rd: bool,
    #[serde(rename="RA")]
    ra: bool,
    #[serde(rename="AD")]
    ad: bool,
    #[serde(rename="CD")]
    cd: bool,
    #[serde(rename="Question")]
    question: Question,
    #[serde(rename="Answer")]
    answer: Option<Vec<Answer>>,
    #[serde(rename="Comment")]
    comment: Option<String>,
}
