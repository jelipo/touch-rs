use async_std::io;
use async_std::io::ReadExt;
use async_std::net::TcpStream;
use async_std::prelude::*;
use async_std::future;
use async_std::future::Future;

use crate::net::proxy::{OutputProxy, ProxyInfo, ProxyReader, ProxyWriter};

pub async fn new_proxy(out_proxy: &mut Box<dyn OutputProxy>, input_stream: &mut TcpStream, info: ProxyInfo) -> io::Result<()> {


    let b = future::ready(1u8);
    let c = future::ready(2u8);

    let f = b.race(c).await;
    Ok(())
}

async fn read(mut input_read: TcpStream, mut out_writer: Box<dyn ProxyWriter + Send>) -> io::Result<()> {
    let mut buf = [0u8; 1024];
    loop {
        let size = input_read.read(&mut buf).await?;
        if size == 0 { break; }
        out_writer.write(&buf[0..size]).await?;
    }
    Ok(())
}

async fn write(mut input_write: TcpStream, mut out_reader: Box<dyn ProxyReader + Send>) -> io::Result<()> {
    loop {
        let data = out_reader.read().await?;
        if data.len() == 0 { break; }
        input_write.write_all(data.as_slice()).await?;
    }
    Ok(())
}