//https://serde.rs/attr-default.html
//#[serde(rename="dns_api_server_name", default="default_api_server_name")]
//pub api_server_name: String,
//#[serde(rename="dns_api_server_addr", default="default_api_server_addr")]
//pub api_server_addr: SocketAddr,
//fn default_api_server_name() -> String { "dns.google.com".to_string() }
//pub cpu_pool: usize,
//fn default_api_server_addr() -> SocketAddr { "4.31.115.251:443".to_socket_addrs().unwrap().next().unwrap() }
//fn default_cpu_pool() -> usize { 4 }

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigToml {
    pub config: Config
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    //listening_addr
    #[serde(rename="listening_addr")]
    pub listening_addr: SocketAddr,

    //name during the handshake & GET request (may be split into two parameters later)
    #[serde(rename="dns_api_server_name")]
    pub api_server_name: String,

    //ip of the server we connect to (this will also be resolved if its an address,
    //but then you can't replace the system DNS server)
    #[serde(rename="dns_api_server_addr")]
    pub api_server_addr: SocketAddr,

    #[serde(rename="dns_api_server_cert_file_path")]
    pub api_cert_path: String,

    //#[serde(default="default_cpu_pool")]
    pub cpu_pool: usize,
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
