use mio::net::TcpListener;
use std::future::Future;
use crate::Stream;
use std::pin::Pin;
use std::task::Context;
use crate::execution::state::execute::current_state;
use crate::execution::current_task;

pub struct AcceptFuture<'a> {
    listener : &'a mut TcpListener,
    registered : bool,
}

impl<'a> AcceptFuture<'a> {
    pub fn new(listener : &'a mut TcpListener) -> Self {
        Self {
            listener,
            registered : false,
        }
    }
}

impl Future for AcceptFuture<'_> {
    type Output = Result<Stream, std::io::Error>;
    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> std::task::Poll<Self::Output> {
        let state_ptr = current_state();
        let state = unsafe { &mut *state_ptr };
        loop {
            let tcp_stream = self.listener.accept();
            match tcp_stream {
                Ok(stream) => {
                    if self.registered {
                        state.pl.deregister(self.listener);
                        self.registered = false;
                    }
                    return std::task::Poll::Ready(Ok(Stream::new(stream.0)));
                }
                Err(error) => {
                    match error.kind() {
                        std::io::ErrorKind::WouldBlock => {
                            if !self.registered {
                                state.pl.register(self.listener, current_task(), mio::Interest::READABLE);
                                self.registered = true;
                            }
                            return std::task::Poll::Pending;
                        }
                        std::io::ErrorKind::Interrupted => {
                            continue;
                        }
                        _ => {
                            if self.registered {
                                state.pl.deregister(self.listener);
                                self.registered = false;
                            }
                            return std::task::Poll::Ready(Err(error));
                        }
                    }
                }
            }
        }
    }
}