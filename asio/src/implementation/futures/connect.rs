

use mio::net::TcpStream;
use std::net::SocketAddr;

use crate::{Stream, Connector};



// connecting never blocks

impl Connector {
    pub fn connect(address : &str) -> Result<Stream, std::io::Error> {
        let addr : SocketAddr = address.parse().expect("Couldn't read the address");
        let tcp_stream = TcpStream::connect(addr)?;
        Ok(Stream::new(tcp_stream))
    }
}