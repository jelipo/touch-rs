use async_std::io;
use async_std::net::{TcpListener, TcpStream};
use async_std::prelude::*;
use async_std::task;

use crate::socks::socks5::{Connect, Socks5};

mod socks;

fn main() -> io::Result<()> {
    task::block_on(async {
        let listener = TcpListener::bind("127.0.0.1:18081").await.unwrap();
        println!("Listening on {}", listener.local_addr().unwrap());

        let mut incoming = listener.incoming();

        while let Some(stream) = incoming.next().await {
            let stream = stream.unwrap();
            task::spawn(async move {
                let socks5 = Socks5::new(stream);
                socks5.connect();
            });
        }
        Ok(())
    })
}

async fn process(stream: TcpStream) -> io::Result<()> {
    println!("Accepted from: {}", stream.peer_addr().unwrap());

    let mut reader = stream.clone();
    let mut writer = stream;
    println!("{:?}", std::thread::current().id());
    let mut arr = [0u8; 4];
    while let read = reader.read(&mut arr).await {
        let i = read.unwrap();
        if i == 0 { break; }
        println!("{:?}", arr);
    }
    println!("{:?}", "你好".as_bytes());
    Ok(())
}