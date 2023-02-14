

use mio::net::TcpStream;
use std::{net::SocketAddr};

// connecting never blocks but it just saves the user some cognitive hassle if we hide
// that technicality from them.
#[allow(dead_code)]
pub fn connect(addr: & SocketAddr) -> Result<TcpStream, std::io::Error> {
    TcpStream::connect(*addr)
}
