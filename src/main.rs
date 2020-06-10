use async_std::io;
use async_std::io::Cursor;
use async_std::net::{TcpListener, TcpStream};
use async_std::prelude::*;
use async_std::task;

use crate::socks::socks5::Socks5;

mod socks;

fn main() -> io::Result<()> {
    task::block_on(async {
        let listener = TcpListener::bind("127.0.0.1:10801").await.unwrap();
        println!("Listening on {}", listener.local_addr().unwrap());

        let mut incoming = listener.incoming();

        while let Some(stream) = incoming.next().await {
            let mut client_stream = stream.unwrap();
            task::spawn(async move {
                let mut socks5 = Socks5::new(&mut client_stream);
                match socks5.connect().await {
                    Some(remote_stream) => {
                        proxy(&mut client_stream, &remote_stream).await
                    }
                    _ => {}
                }
                println!("结束")
            });
        }
        Ok(())
    })
}

async fn proxy(client_stream: &mut TcpStream, remote_stream: &TcpStream) {

    let mut client_read = client_stream.clone();
    let mut client_write = client_stream.clone();
    let mut remote_read = remote_stream.clone();
    let mut remote_write = remote_stream.clone();
    loop {

    }
    let handle1 = task::spawn(async move {
        io::copy(&mut client_read, &mut remote_write).await
    });
    let handle2 = task::spawn(async move {
        io::copy(&mut remote_read, &mut client_write).await
    });
    let result1 = handle1.await;
    match result1 {
        Ok(re) => {
            println!("{}", re)
        }
        Err(e) => {
            eprintln!(" {}", e);
        }
    }

    let result2 = handle2.await;
    match result2 {
        Ok(re) => {
            println!("1 {}", re)
        }
        Err(e) => {
            eprintln!("2 {}", e);
        }
    }
}