
use super::{*, write::WriteFuture};
use self::read::*;
use mio::net::TcpStream;
use std::cell::RefCell;
use crate::execution::accept::AcceptFuture;

pub struct Listener {
    listener : mio::net::TcpListener,
}

impl Listener {
    pub fn bind(address : &str) -> std::io::Result<Self> {
        let listener = mio::net::TcpListener::bind(address.parse().expect("Couldn't read the address"))?;
        Ok(Listener { listener } )
    }

    pub fn accept(&mut self) -> AcceptFuture {
        AcceptFuture::new(&mut self.listener)
    }
}

pub struct Stream {
    stream : RefCell<TcpStream>,
}

unsafe impl Sync for Stream {} // future should never be polled by more than one thread
unsafe impl Send for Stream {} // future should never be polled by more than one thread

impl<'a> Stream {
    pub(super) fn new(stream : TcpStream) -> Self {
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

    pub async fn write_all(&self, buffer : &[u8]) -> Result<usize, std::io::Error> {
        let mut byte_index = 0;
        while byte_index < buffer.len() {
            let write_result = self.write(&buffer[byte_index..]).await;
            match write_result {
                Ok(bytes_written) => byte_index += bytes_written,
                Err(_) => return write_result
            }
        }
        Ok(byte_index)
    }
}





