use mio::net::TcpListener;
use std::{io::{self, ErrorKind}, task::{Context, Poll}, pin::Pin, future::Future};
use crate::{implementation::{ state::{taskqueue::current_state, poll::NetFD, blockingstate::State}, task::current_task}, Listener, Stream};
use std::io::Error;

impl Listener {
    pub fn bind(address : &str) -> io::Result<Self> {
        let listener = mio::net::TcpListener::bind(address.parse().expect("Couldn't read the address"))?;
        Ok(Listener { listener } )
    }

    pub fn accept(&mut self) -> AcceptFuture {
        AcceptFuture::new(&mut self.listener)
    }
}

pub struct AcceptFuture<'a> {
    listener : &'a mut TcpListener,
}

impl<'a> AcceptFuture<'a> {
    pub(crate) fn new(listener : &'a mut TcpListener) -> Self {
        Self {
            listener,
        }
    }

    fn register(mut self: Pin<&mut Self>, state: &mut State) {
        state.pl.register(NetFD::Listener(&mut *self.listener), current_task(), mio::Interest::READABLE);
    }
    
}

impl Future for AcceptFuture<'_> {
    type Output = Result<Stream, Error>;
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> std::task::Poll<Self::Output> {
        // executors must live longer than their tasks
        let state = unsafe { current_state() } ;
        loop {
            let tcp_stream = self.listener.accept();
            match tcp_stream {
                Ok(stream) => {
                    return Poll::Ready(Ok(Stream::new(stream.0)));
                }
                Err(error) => {
                    match error.kind() {
                        ErrorKind::WouldBlock => {
                            self.register(state);
                            return Poll::Pending;
                        }
                        ErrorKind::Interrupted => {
                            continue;
                        }
                        _ => {
                            return Poll::Ready(Err(error));
                        }
                    }
                }
            }
        }
    }
}



