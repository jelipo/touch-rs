
use async_std::io;
use async_std::net::{Shutdown, TcpListener, TcpStream};
use async_std::prelude::*;
use async_std::sync::Mutex;
use async_std::task;
use cfb_mode::Cfb;
use cfb_mode::stream_cipher::{NewStreamCipher, StreamCipher};
use fantasy_util::time::system_time::SystemLocalTime;

use crate::socks::consts::SocksVersion;
use crate::socks::socks5_connector::Socks5Connector;

mod socks;
mod ss;
mod encrypt;



fn main() {


    //
    task::block_on(start());
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
                Err(err) => {
                    eprintln!("创建连接失败,异常信息:{}", err);
                }
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
    let handle1 = task::spawn(async move {
        match io::copy(&mut client_read, &mut remote_write).await {
            Ok(size) => println!("从client收到:{} byte {}", size, id),
            _ => {}
        };
        client_read.shutdown(Shutdown::Both);
        remote_write.shutdown(Shutdown::Both);
    });
    let handle2 = task::spawn(async move {
        match io::copy(&mut remote_read, &mut client_write).await {
            Ok(size) => println!("从remote收到:{} byte {}", size, id),
            _ => {}
        }
        client_write.shutdown(Shutdown::Both);
        remote_read.shutdown(Shutdown::Both);
    });
    handle2.await;
    client_stream.shutdown(Shutdown::Both);
    handle1.await;
    let mutex = Mutex::new("1");
    mutex.lock();
}
