
use asio::{ Stream, Listener, AsyncMutex, async_channel, RingBuffer, async_spawn };
use std::{ sync::Arc, fs };

async fn client(stream : Stream, mutex : Arc<AsyncMutex<usize>>) {
    let mut buffer = [0u8; 1024];

    
    
    let result = stream.read(&mut buffer).await;
    match result {
        Ok(bytes_read) => {
            for byte in buffer[..bytes_read].iter() {
                print!("{}", *byte as char);
            }
            println!();
        }
        Err(_) => {
            println!("error");
            return;
        }
    }

    // string from https://rosettacode.org/wiki/Hello_world/Web_server#Rust
    let response =
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n
        <!DOCTYPE html><html><head><title>Bye-bye baby bye-bye</title>
        <style>body { background-color: #111 }
        h1 { font-size:4cm; text-align: center; color: black;
        text-shadow: 0 0 2mm red}</style></head>
        <body><h1>Goodbye, world!</h1></body></html>\r\n".as_bytes();

    // async mutex usage
    let mut var = mutex.lock().await;
    *var += 1;
    println!("client number: {}", *var);
    
    stream.write_all(response).await.unwrap();
}


async fn async_main() {
    // Kitchen sink
    let mx : Arc<AsyncMutex<usize>> = Arc::new(AsyncMutex::new(0));
    let (sdr, rvr) = async_channel();
    let _ = sdr.send(4);
    let _ = sdr.send(4);
    let _ = rvr.receive().await;
    let _ = rvr.try_receive();
    let aa = RingBuffer::new(10);
    aa.send(4).await;
    let _ = aa.recv().await;

    // TCP event loop
    let ip = fs::read_to_string("src/config.txt").expect("Unable to read address");
    let mut listener = Listener::bind(&ip).expect("Couldn't bind to port");
    while let Ok(stream) = listener.accept().await {
        async_spawn(
            client(stream, mx.clone())
        );
    }
}

fn main() {
    asio::runtime(async_main());
}

