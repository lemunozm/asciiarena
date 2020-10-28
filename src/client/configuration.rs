use std::net::{SocketAddr};

pub struct Config {
    pub server_addr: Option<SocketAddr>,
    pub character: Option<char>,
}
