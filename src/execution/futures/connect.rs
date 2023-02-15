

use mio::net::TcpStream;
use std::net::SocketAddr;

// connecting never blocks
#[allow(dead_code)]
pub fn connect(addr: & SocketAddr) -> Result<TcpStream, std::io::Error> {
    TcpStream::connect(*addr)
}
