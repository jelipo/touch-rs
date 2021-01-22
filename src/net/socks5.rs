use std::str::FromStr;


use log::{error, info};

use crate::core::profile::BasePassiveConfig;
use crate::net::proxy::{Closer, InputProxy, OutProxyStarter, OutputProxy, ProxyReader, ProxyWriter};
use crate::socks::socks5_connector::Socks5Connector;
use std::io::{Error, ErrorKind};
use std::net::{SocketAddr, Shutdown};
use tokio::net::{TcpListener, TcpStream};
use std::io;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use std::sync::Arc;
use std::cell::RefCell;
use std::ops::{DerefMut, Deref};
use tokio::net::tcp::{WriteHalf, ReadHalf};


pub struct Socks5Passive {
    tcp_listener: TcpListener,
    password: Option<String>,
    out_proxy: Box<dyn OutputProxy + Send>,
}

impl Socks5Passive {
    /// Init Socks5 Passive. And try to bind host and port
    pub async fn new(passive: &BasePassiveConfig, out_proxy: Box<dyn OutputProxy + Send>) -> io::Result<Self> {
        let addr_str = format!("{}:{}", &passive.local_host, passive.local_port);
        let addr = SocketAddr::from_str(addr_str.as_str()).or(
            Err(Error::new(ErrorKind::InvalidInput, "Error address"))
        );
        let tcp_listener = TcpListener::bind(addr?).await?;
        info!("Socks5 bind in {}", addr_str);
        Ok(Self {
            tcp_listener,
            password: passive.password.clone(),
            out_proxy,
        })
    }
}

#[async_trait]
impl InputProxy for Socks5Passive {
    async fn start(&mut self) -> io::Result<()> {
        info!("Sock5 start listen");
        loop {
            let out_proxy = &mut self.out_proxy;
            let (mut tcpstream, addr) = self.tcp_listener.accept().await?;
            let starter = match out_proxy.gen_starter() {
                Ok(n) => n,
                Err(_) => continue
            };
            tokio::task::spawn(async move {
                if let Err(e) = new_proxy(tcpstream, starter).await {
                    error!("Socks5 proxy error. {}", e)
                };
            });
        }
    }
}


async fn new_proxy(mut input_stream: TcpStream, mut starter: Box<dyn OutProxyStarter>) -> io::Result<()> {
    let mut connector = Socks5Connector::new(&mut input_stream);
    let info = connector.check().await?;

    let (out_reader,
        out_writer,
        mut closer) = starter.new_connect(info).await?;
    let (read_half, write_half) = input_stream.split();
    let reader = async {
        write(write_half, out_reader).await
    };
    let writer = async {
        read(read_half, out_writer).await
    };
    // Wait for two future done.
    tokio::select! {
        _ = reader => {}
        _ = writer => {}
    }
    let _sd_rs = input_stream.shutdown().await;
    let _closer_rs = closer.shutdown().await;
    Ok(())
}

async fn read(mut input_read: ReadHalf, mut out_writer: Box<dyn ProxyWriter>) -> usize {
    let mut buf = [0u8; 1024];
    let mut total = 0;
    while let Ok(size) = input_read.read(&mut buf).await {
        if size == 0 { break; }
        total = total + size;
        if out_writer.write(&mut buf[..size]).await.is_err() { break; }
    }
    total
}

async fn write(mut input_write: WriteHalf, mut out_reader: Box<dyn ProxyReader>) -> usize {
    let mut total = 0;
    while let Ok(data) = out_reader.read().await {
        if data.len() == 0 { break; }
        total = total + data.len();
        if input_write.write_all(data.as_ref()).await.is_err() { break; };
    }
    total
}