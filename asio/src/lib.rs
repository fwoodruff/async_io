pub(crate) mod implementation;
pub mod mpsc;

use std::{cell::{RefCell, UnsafeCell}, sync::atomic::AtomicUsize, future::Future};

use implementation::{task::TaskID, futures::{mutex::AsyncMutexInternal, mpscimpl::{AsyncSender, async_channel_impl}}};
use mio::net::TcpStream;
use mpsc::AsyncReceiver;
use crate::implementation::{ async_spawn_impl, runtime_impl};

pub struct Listener {
    listener : mio::net::TcpListener,
}
pub struct Stream {
    stream : RefCell<TcpStream>,
}

pub struct Connector;

pub struct JoinFuture {
    child : TaskID
}

pub struct AsyncMutex<T> {
    lock : AsyncMutexInternal,
    data : UnsafeCell<T>,
}

pub fn async_channel<T>() -> (AsyncSender<T>, AsyncReceiver<T>) {
    async_channel_impl()
}
pub struct RingBuffer<T> {
    buffer : Vec<RefCell<Option<T>>>,
    num_elems : AtomicUsize,
    insertion_idx : AtomicUsize,
    removal_idx : AtomicUsize,
}

pub fn async_spawn(f: impl Future<Output = ()> + Send + 'static) -> JoinFuture {
    async_spawn_impl(f)
}

 // Entry point for user-provided async main
 pub fn runtime(main_task: impl Future<Output = ()> + Send + 'static) {
    runtime_impl( main_task )
}



#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use super::*;

    async fn async_kitchen_sink() {
        let mutex : Arc<AsyncMutex<usize>> = Arc::new(AsyncMutex::new(0));
        let (sdr, rvr) = async_channel();
        let _ = sdr.send(4);
        let _ = sdr.send(4);
        let _ = rvr.receive().await;
        let _ = rvr.try_receive();
        let aa = RingBuffer::new(10);
        aa.send(4).await;
        let _ = aa.recv().await;
       
        let mut var = mutex.lock().await;
        *var += 1;
        println!("client number: {}", *var);
    }

    async fn _async_test_io() {
        let addr = "[::]:9410";
        let hello = "GET / 1.1\r\nHost: www.example.com\r\n\r\n".as_bytes();
        let mut listener = Listener::bind(addr).expect("Couldn't bind to port");
        let client_stream = Connector::connect(addr).expect("connect failed");
        let tm : Result<Stream, std::io::Error> = listener.accept().await;
        let server_stream : Stream = tm.unwrap();
        let _ = client_stream.write(hello).await;
        let mut buffer = [0u8; 1024];
        let _ = server_stream.read(&mut buffer[..]).await;
    }

    #[test]
    fn test_io() {
        implementation::runtime(_async_test_io());
    }

    #[test]
    fn kitchen_sink() {
        implementation::runtime(async_kitchen_sink());
    }
    
}


