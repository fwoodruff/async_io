
mod execution;
use crate::execution::{futures::listen::{Listener, Stream}, async_spawn};
use std::fs;

async fn client(stream : Stream) {
    let mut buff = [0u8; 1024];
    let res = stream.read(&mut buff).await;
    match res {
        Ok(result) => {
            for byte in buff[..result].iter() {
                print!("{}", *byte as char);
            }
            println!("");
        }
        Err(_) => {
            return;
        }
    }

    // https://rosettacode.org/wiki/Hello_world/Web_server#Rust
    let response =
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n
        <!DOCTYPE html><html><head><title>Bye-bye baby bye-bye</title>
        <style>body { background-color: #111 }
        h1 { font-size:4cm; text-align: center; color: black;
        text-shadow: 0 0 2mm red}</style></head>
        <body><h1>Goodbye, world!</h1></body></html>\r\n".as_bytes();

        if let Err(_) = stream.write_all(response).await {
            return;
        }
    
}

async fn async_main() {
    let ip = fs::read_to_string("src/config.txt").expect("Unable to read file");
    let mut listener = Listener::bind(&ip).unwrap();
    while let Ok(stream) = listener.accept().await {
        async_spawn(
            client(stream)
        );
    }
}

fn main() {
    execution::runtime(async_main());
}