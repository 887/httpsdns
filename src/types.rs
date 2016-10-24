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
    pub questions: Vec<Question>,
    #[serde(rename="Answer")]
    pub answers: Option<Vec<Answer>>,
    #[serde(rename="Comment")]
    pub comment: Option<String>,
}

impl Answer {
    pub fn write(&self) -> Result<Vec<u8>, ()> {
        use std::net::{Ipv4Addr, Ipv6Addr};
        use std::str::FromStr;

        match self.atype {
            1  =>  {
                let ip = Ipv4Addr::from_str(&self.data).unwrap();
                Ok(ip.octets().to_vec())
            },
            5  => {
                let mut data : Vec<u8> = Vec::new();
                let name = &self.data;
                //println!("CNAME: {:?}", name);
                for label in name.split('.') {
                    let size = label.len() as u8;
                    data.push(size);
                    data.extend(label.as_bytes());
                }
                Ok(data)
            },
            12 => {
                let mut data : Vec<u8> = Vec::new();
                let name = &self.data;
                for label in name.split('.') {
                    let size = label.len() as u8;
                    data.push(size);
                    data.extend(label.as_bytes());
                }
                Ok(data)
            },
            28 => {
                let ip = Ipv6Addr::from_str(&self.data).unwrap();
                let mut ipv6_bytes : Vec<u8> = Vec::new();
                for segment in ip.segments().iter() {
                    let upper = segment >> 8;
                    let lower = segment & 0b0000_0000_1111_1111;
                    ipv6_bytes.push(upper as u8);
                    ipv6_bytes.push(lower as u8);
                }
                Ok(ipv6_bytes)
            }
            _ => Err(())
        }
    }
}
