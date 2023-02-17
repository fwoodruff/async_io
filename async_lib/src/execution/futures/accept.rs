use mio::net::TcpListener;
use std::future::Future;
use std::io::ErrorKind;
use std::io::Error;
use std::task::Poll;
use crate::execution::state::poll::NetFD;
use crate::execution::state::taskqueue::current_state;
use std::pin::Pin;
use std::task::Context;
use crate::execution::State;

use crate::execution::current_task;

use super::listen::Stream;

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



