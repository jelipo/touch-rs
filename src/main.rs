use std::path::Path;

use async_std::io;
use async_std::net::{Shutdown, TcpListener, TcpStream};
use async_std::prelude::*;
use async_std::task;
use async_std::task::JoinHandle;

use crate::core::config::ConfigReader;
use crate::core::selector::ProtocalSelector;
use crate::encrypt::aead::AeadType;
use crate::net::proxy::ProxyReader;
use crate::net::ss_stream::SsStreamReader;
use crate::socks::socks5::Socks5;

mod socks;
mod ss;
mod encrypt;
mod net;
mod core;
mod util;

#[async_std::main]
async fn main() -> io::Result<()> {
    env_logger::init();

    //listen().await

    let path = Path::new("./conf/config.json");
    let reader = ConfigReader::read_config(path)?;
    ProtocalSelector::select(&reader).await
}

async fn listen() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:3391").await.unwrap();
    let mut incoming = listener.incoming();
    let option: Option<io::Result<TcpStream>> = incoming.next().await;
    let mut stream = option.unwrap().unwrap();

    let mut reader = SsStreamReader::new(stream, "test", AeadType::AES256GCM);
    let vec = reader.read().await?;
    println!("Read:{:?}", vec);
    let de_data = vec.as_slice();
    let addrs = Socks5::read_to_socket_addrs(de_data);
    println!("addr {:?}", addrs);
    let data = &de_data[(addrs.1)..de_data.len()];
    println!("{:?}", String::from_utf8(data.to_vec()).unwrap().as_str());
    Ok(())
}

async fn start() -> io::Result<()> {
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
}