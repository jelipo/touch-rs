use std::io::{Error, ErrorKind};
use std::io;
use std::net::SocketAddr;
use std::str::FromStr;

use async_trait::async_trait;
use log::{error, info};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

use crate::core::profile::BasePassiveConfig;
use crate::net::proxy::{InputProxy, OutProxyStarter, OutputProxy, ProxyReader, ProxyWriter};
use crate::socks::socks5_connector::Socks5Connector;

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
            let (tcpstream, _addr) = self.tcp_listener.accept().await?;
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

    let (mut out_reader, mut out_writer) = starter.new_connect(info).await?;

    let (read_half, write_half) = input_stream.into_split();
    let reader = write(write_half, &mut out_reader);
    let writer = read(read_half, &mut out_writer);
    // Wait for two future done.
    tokio::select! {
        _ = reader => {}
        _ = writer => {}
    }
    // TODO Don't know TCP will be dropped.
    // let _sd_rs = input_stream.shutdown().await;
    Ok(())
}

async fn read(mut input_read: OwnedReadHalf, out_writer: &mut Box<dyn ProxyWriter>) -> usize {
    let mut buf = [0u8; 1024];
    let mut total = 0;
    while let Ok(size) = input_read.read(&mut buf).await {
        if size == 0 { break; }
        total = total + size;
        if out_writer.write(&mut buf[..size]).await.is_err() { break; }
    }
    total
}

async fn write(mut input_write: OwnedWriteHalf, out_reader: &mut Box<dyn ProxyReader>) -> usize {
    let mut total = 0;
    while let Ok(data) = out_reader.read().await {
        if data.len() == 0 { break; }
        total = total + data.len();
        if input_write.write_all(data.as_ref()).await.is_err() { break; };
    }
    let _result = input_write.shutdown().await;
    total
}