
use super::super::*;
use crate::execution::state::taskqueue::current_state;
use crate::execution::state::poll::NetFD;

pub struct WriteFuture<'a> {
    buff : &'a [u8],
    stream : &'a RefCell<TcpStream>,
}

unsafe impl Send for WriteFuture<'_> {} // future has a single owner when dropped or polled

impl<'a> WriteFuture<'a> {
    pub(super)
    fn new(buff : &'a [u8], stream : &'a RefCell<TcpStream>) -> Self {
        Self {
            buff,
            stream,
        }
    }

    // add write request to polling context
    fn register(self: Pin<&mut Self>, state: &mut State, stream_ref: &mut TcpStream) {
        state.pl.register(NetFD::Stream(stream_ref), current_task(), mio::Interest::READABLE);
        
    }
}

impl Future for WriteFuture<'_> {
    type Output = Result<usize, std::io::Error>;

    // if this wakes up that means the polling context is happy for us to write to this socket
    fn poll(self: Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        loop {
            let mut stream_borrow = self.stream.borrow_mut();
            let mut stream_ref = stream_borrow.by_ref();
            let res = std::io::Write::write(&mut stream_ref, self.buff);
            // executors must live longer than their tasks
            let state = unsafe {current_state() } ;
            

            match res {
                Ok(_) => {
                    
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
                            
                            return std::task::Poll::Ready(res);
                        }
                    }
                }
            }
        }
    }
}



