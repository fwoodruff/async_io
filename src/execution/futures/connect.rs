

use mio::net::TcpStream;
use std::{net::SocketAddr, future::Future, task::{Context, Poll}};

// connecting never blocks but it just saves the user some cognitive hassle if we hide
// that technicality from them.
#[allow(dead_code)]
fn connect(addr: & SocketAddr) -> Result<TcpStream, std::io::Error> {
    TcpStream::connect(*addr)
}

pub struct ConnectFuture<'a> {
    addr: &'a SocketAddr,
}

impl<'a> Future for ConnectFuture<'a> {
    type Output = Result<TcpStream, std::io::Error>;

    fn poll(self: std::pin::Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let result = TcpStream::connect(*self.addr);
        Poll::Ready(result)
    }
}
