use mio::net::TcpListener;
use std::future::Future;
use crate::Stream;
use crate::execution::state::poll::NetFD;
use std::pin::Pin;
use std::task::Context;
use crate::execution::state::execute::current_state;
use crate::execution::current_task;

pub struct AcceptFuture<'a> {
    listener : &'a mut TcpListener,
}

impl<'a> AcceptFuture<'a> {
    pub(in crate::execution::futures) fn new(listener : &'a mut TcpListener) -> Self {
        Self {
            listener,
        }
    }

    fn register(mut self: Pin<&mut Self>, state: &mut crate::execution::state::State) {
        state.pl.register(NetFD::Listener(&mut *self.listener), current_task(), mio::Interest::READABLE);
    }
    
}

impl Future for AcceptFuture<'_> {
    type Output = Result<Stream, std::io::Error>;
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> std::task::Poll<Self::Output> {
        // executors must live longer than their tasks
        let state = unsafe { current_state() } ;
        loop {
            let tcp_stream = self.listener.accept();
            match tcp_stream {
                Ok(stream) => {
                    return std::task::Poll::Ready(Ok(Stream::new(stream.0)));
                }
                Err(error) => {
                    match error.kind() {
                        std::io::ErrorKind::WouldBlock => {
                            self.register(state);
                            return std::task::Poll::Pending;
                        }
                        std::io::ErrorKind::Interrupted => {
                            continue;
                        }
                        _ => {
                            return std::task::Poll::Ready(Err(error));
                        }
                    }
                }
            }
        }
    }
}



