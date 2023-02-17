

use mio::net::TcpStream;
use std::net::SocketAddr;

use super::listen::Stream;

// connecting never blocks
#[allow(dead_code)]
pub fn connect(address : &str) -> Result<Stream, std::io::Error> {
    let addr : SocketAddr = address.parse().expect("Couldn't read the address");
    let tcp_stream = TcpStream::connect(addr)?;
    Ok(Stream::new(tcp_stream))
}
