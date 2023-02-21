
use std::{task::{Poll, Context}, io::{ErrorKind, Read, self, Error}, cell::RefCell, pin::Pin, future::Future};
use mio::net::TcpStream;
use crate::{implementation::{state::{taskqueue::current_state, poll::NetFD, blockingstate::State}, task::current_task}, Stream};


unsafe impl Sync for Stream {} // future should never be polled by more than one thread
unsafe impl Send for Stream {} // future should never be polled by more than one thread




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
    type Output = Result<usize, io::Error>;

    // if this wakes up that means the polling context is happy for us to write to this socket
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            let mut stream_borrow = self.stream.borrow_mut();
            let mut stream_ref = stream_borrow.by_ref();
            let res = std::io::Write::write(&mut stream_ref, self.buff);
            // executors must live longer than their tasks
            let state = unsafe { current_state() } ;

            match res {
                Ok(_) => {
                    return Poll::Ready(res);
                }
                Err(ref error) => {
                    match error.kind() {
                        ErrorKind::WouldBlock => {
                            self.register(state, stream_ref);
                            return Poll::Pending;
                        }
                        ErrorKind::Interrupted => {
                            continue;
                        }
                        _ => {
                            return Poll::Ready(res);
                        }
                    }
                }
            }
        }
    }
}


impl<'a> Stream {
    pub(crate) fn new(stream : TcpStream) -> Self {
        Stream {
            stream : RefCell::new(stream),
        }
    }

    pub fn read(&'a self, buffer : &'a mut [u8] ) -> ReadFuture<'a> {
        ReadFuture::new(buffer, &self.stream)
    }

    pub fn write(&'a self, buffer : &'a [u8]) -> WriteFuture<'a> {
        WriteFuture::new(buffer, &self.stream)
    }

    pub async fn write_all(&'a self, buffer : &'a [u8]) -> Result<(), Error> {
        let mut byte_index = 0;
        while byte_index < buffer.len() {
            let write_result = self.write(&buffer[byte_index..]).await;
            match write_result {
                Ok(bytes_written) => byte_index += bytes_written,
                Err(err) => return Err(err)
            }
        }
        Ok(())
    }
}



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
    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            let mut stream_borrow = self.stream.borrow_mut();
            let stream_ref = stream_borrow.by_ref();
            let res = stream_ref.read(self.buff);
            // executors must live longer than their tasks
            let state = unsafe { current_state() } ;
            
            match res {
                Ok(_) => {
                    return Poll::Ready(res);
                }
                Err(ref error) => {
                    match error.kind() {
                        ErrorKind::WouldBlock => {
                            state.pl.register(NetFD::Stream(stream_ref), current_task(), mio::Interest::READABLE);
                            return Poll::Pending;
                        }
                        ErrorKind::Interrupted => {
                            continue;
                        }
                        _ => {
                            return Poll::Ready(res);
                        }
                    }
                }
            }
        }
    }
}