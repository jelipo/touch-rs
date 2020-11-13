use async_std::io;
use async_std::net::{Shutdown, TcpListener, TcpStream};
use async_std::prelude::*;
use async_std::sync::Mutex;
use async_std::task;
use async_std::task::JoinHandle;
use fantasy_util::time::system_time::SystemLocalTime;

use crate::encrypt::aead::AeadType;
use crate::net::stream::{SsStreamReader, StreamReader};
use crate::socks::socks5::Socks5;
use crate::socks::socks5_connector::Socks5Connector;

mod socks;
mod ss;
mod encrypt;
mod net;
mod core;


fn main() -> io::Result<()> {

    //
    task::block_on(listen())

    //task::block_on(start())
}

async fn listen() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:3391").await.unwrap();
    let mut incoming = listener.incoming();
    let option = incoming.next().await;
    let mut stream = option.unwrap().unwrap();
    let mut reader = SsStreamReader::new(&mut stream, b"test", &AeadType::AES256GCM);
    let vec = reader.read().await?;
    println!("Read:{:?}", vec);
    let de_data = vec.as_slice();
    let addrs = Socks5::read_to_socket_addrs(de_data);
    println!("{:?}", addrs);
    let data = &de_data[(addrs.1)..de_data.len()];
    println!("{:?}", String::from_utf8(data.to_vec()).unwrap().as_str());
    Ok(())
}

async fn start() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:10801").await.unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());
    let mut incoming = listener.incoming();
    while let Some(stream) = incoming.next().await {
        let mut client_stream = stream.unwrap();
        task::spawn(async move {
            let id = SystemLocalTime::unix_mills();
            println!("开始创建连接{}", id);
            let mut socks5 = Socks5Connector::new(&mut client_stream);
            match socks5.connect().await {
                Ok(mut remote_stream) => {
                    proxy(&mut client_stream, &mut remote_stream, id).await
                }
                Err(err) => eprintln!("创建连接失败,异常信息:{}", err)
            }
            println!("此连接生命周期结束")
        });
    }
    Ok(())
}

async fn proxy(client_stream: &mut TcpStream, remote_stream: &mut TcpStream, id: u64) {
    let mut client_read = client_stream.clone();
    let mut client_write = client_stream.clone();
    let mut remote_read = remote_stream.clone();
    let mut remote_write = remote_stream.clone();

    let handle1: JoinHandle<u64> = task::spawn(async move {
        return io::copy(&mut client_read, &mut remote_write).await.unwrap();
    });
    let handle2: JoinHandle<u64> = task::spawn(async move {
        return io::copy(&mut remote_read, &mut client_write).await.unwrap();
    });

    // TODO 阻塞等待完成
    handle2.await;
    client_stream.shutdown(Shutdown::Both);
    remote_stream.shutdown(Shutdown::Both);
    handle1.await;
    let mutex = Mutex::new("1");
    mutex.lock();
}