
use super::super::*;
use crate::execution::state::execute::current_state;

pub struct ReadFuture<'a> {
    buff : &'a mut [u8],
    stream : &'a RefCell<TcpStream>,
    registered : bool,

}
unsafe impl Send for ReadFuture<'_> {} // future has a single owner when dropped or polled

impl<'a> ReadFuture<'a> {
    pub fn new(buff : &'a mut [u8], stream : &'a RefCell<TcpStream>) -> Self {
        Self {
            buff,
            stream,
            registered : false,
        }
    }
}



impl Future for ReadFuture<'_> {
    type Output = Result<usize, std::io::Error>;
    fn poll(mut self: Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        loop {
            let mut stream_borrow = self.stream.borrow_mut();
            let stream_ref = stream_borrow.by_ref();
            let res = stream_ref.read(self.buff);
            crate::readwriteresult!(res, self, stream_ref);
        }
    }
}