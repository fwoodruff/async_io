

use mio::net::TcpStream;
use std::net::SocketAddr;

// connecting never blocks so no need to await
#[allow(dead_code)]
fn connect(addr: & SocketAddr) -> Result<TcpStream, std::io::Error> {
    TcpStream::connect(*addr)
}

