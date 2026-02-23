use crate::constants::{UDP_DEFAULT_LISTEN_PORT, UDP_MAX_PACKET_SIZE};

use std::io;
use std::net::UdpSocket;

pub struct Config {
    pub host: String,
    pub port: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: UDP_DEFAULT_LISTEN_PORT,
        }
    }
}

pub fn listen(config: Config) -> io::Result<()> {
    let socket_str = format!("{}:{}", config.host, config.port);
    let socket = UdpSocket::bind(&socket_str)?;
    println!("Listener started on on: {}", &socket_str);

    let mut buf = [0; UDP_MAX_PACKET_SIZE];

    loop {
        let (amt, src) = socket.recv_from(&mut buf)?;
        println!("Received {} bytes from {}", amt, src);
    }
}
