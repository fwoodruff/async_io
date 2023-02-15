
pub mod execution;

#[cfg(test)]
mod tests {
    use crate::execution::futures::{
        listen::{
            Listener, Stream
        },
        mutex::AsyncMutex,
        connect::connect,
        mpsc::async_channel,
        ring_buffer::RingBuffer
    };
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
        let client_stream = connect(addr).expect("connect failed");
        let tm : Result<Stream, std::io::Error> = listener.accept().await;
        let server_stream : Stream = tm.unwrap();
        let _ = client_stream.write(hello).await;
        let mut buffer = [0u8; 1024];
        let _ = server_stream.read(&mut buffer[..]).await;
    }

    #[test]
    fn test_io() {
        execution::runtime(_async_test_io());
    }

    #[test]
    fn kitchen_sink() {
        execution::runtime(async_kitchen_sink());
    }
    
}


