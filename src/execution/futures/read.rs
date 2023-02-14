
use super::super::*;
use crate::execution::state::{taskqueue::current_state, poll::NetFD};

pub struct ReadFuture<'a> {
    buff : &'a mut [u8],
    stream : &'a RefCell<TcpStream>,

}
unsafe impl Send for ReadFuture<'_> {} // future has a single owner when dropped or polled

impl<'a> ReadFuture<'a> {
    pub(super) fn new(buff : &'a mut [u8], stream : &'a RefCell<TcpStream>) -> Self {
        Self {
            buff,
            stream,
        }
    }
}



impl Future for ReadFuture<'_> {
    type Output = Result<usize, std::io::Error>;
    // polling context is happy for us to try reading from the socket
    fn poll(mut self: Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        loop {
            let mut stream_borrow = self.stream.borrow_mut();
            let stream_ref = stream_borrow.by_ref();
            let res = stream_ref.read(self.buff);
            // executors must live longer than their tasks
            let state = unsafe { current_state() } ;
            
            match res {
                Ok(_) => {
                    
                    return std::task::Poll::Ready(res);
                }
                Err(ref error) => {
                    match error.kind() {
                        
                        std::io::ErrorKind::WouldBlock => {
                            state.pl.register(NetFD::Stream(stream_ref), current_task(), mio::Interest::READABLE);
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