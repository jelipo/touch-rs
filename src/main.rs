use std::str;
use std::time::{SystemTime, UNIX_EPOCH};

use async_std::io;
use async_std::io::Cursor;
use async_std::net::{Shutdown, TcpListener, TcpStream};
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
                let id = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
                println!("开始创建连接{}", id);
                let mut socks5 = Socks5::new(&mut client_stream);
                match socks5.connect().await {
                    Some(mut remote_stream) => {
                        proxy(&mut client_stream, &mut remote_stream, id).await
                    }
                    _ => {}
                }
                println!("all结束")
            });
        }
        Ok(())
    })
}

async fn proxy(client_stream: &mut TcpStream, remote_stream: &mut TcpStream, id: u128) {
    let mut client_read = client_stream.clone();
    let mut client_write = client_stream.clone();
    let mut remote_read = remote_stream.clone();
    let mut remote_write = remote_stream.clone();
    let handle1 = task::spawn(async move {
        match io::copy(&mut client_read, &mut remote_write).await {
            Ok(size) => {
                println!("client收到:{} {}", size, id);
            }
            _ => {}
        };
        client_read.shutdown(Shutdown::Both);
        remote_write.shutdown(Shutdown::Both);
    });
    let handle2 = task::spawn(async move {
        match io::copy(&mut remote_read, &mut client_write).await {
            Ok(size) => {
                println!("remote收到:{} {}", size, id);
            }
            _ => {}
        }
        client_write.shutdown(Shutdown::Both);
        remote_read.shutdown(Shutdown::Both);
    });
    handle2.await;
    client_stream.shutdown(Shutdown::Both);
    handle1.await;
    println!("结束2");
}