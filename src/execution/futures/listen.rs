
use super::{*, write::WriteFuture};
use self::read::*;
use mio::net::TcpStream;
use std::{cell::RefCell, io};
use crate::execution::accept::AcceptFuture;
use std::io::Error;

pub struct Listener {
    listener : mio::net::TcpListener,
}

impl Listener {
    pub fn bind(address : &str) -> io::Result<Self> {
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





