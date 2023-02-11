
use super::super::*;
use crate::execution::state::execute::current_state;

pub struct WriteFuture<'a> {
    buff : &'a [u8],
    stream : &'a RefCell<TcpStream>,
    registered : bool,
}

unsafe impl Send for WriteFuture<'_> {} // future has a single owner when dropped or polled

impl<'a> WriteFuture<'a> {
    pub(super)
    fn new(buff : &'a [u8], stream : &'a RefCell<TcpStream>) -> Self {
        Self {
            buff,
            stream,
            registered : false,
        }
    }

    // remove write request from polling context
    fn deregister(mut self: Pin<&mut Self>, state: &mut State, stream_ref: &mut TcpStream) {
        if self.registered {
            state.pl.deregister(stream_ref);
            self.registered = false;
        }
    }
    // add write request to polling context
    fn register(mut self: Pin<&mut Self>, state: &mut State, stream_ref: &mut TcpStream) {
        if !self.registered {
            state.pl.register(stream_ref, current_task(), mio::Interest::READABLE);
            self.registered = true;
        }
    }
}

impl Future for WriteFuture<'_> {
    type Output = Result<usize, std::io::Error>;
    fn poll(self: Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        loop {
            let mut stream_borrow = self.stream.borrow_mut();
            let mut stream_ref = stream_borrow.by_ref();
            let res = std::io::Write::write(&mut stream_ref, self.buff);
            let state_ptr = current_state();
            let state = unsafe { &mut *state_ptr };

            match res {
                Ok(_) => {
                    self.deregister(state, stream_ref);
                    return std::task::Poll::Ready(res);
                }
                Err(ref error) => {
                    match error.kind() {
                        std::io::ErrorKind::WouldBlock => {
                            self.register(state, stream_ref);
                            return std::task::Poll::Pending;
                        }
                        std::io::ErrorKind::Interrupted => {
                            continue;
                        }
                        _ => {
                            self.deregister(state, stream_ref);
                            return std::task::Poll::Ready(res);
                        }
                    }
                }
            }
        }
    }
}



