use std::io::{Error, ErrorKind};
use std::io;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

use async_trait::async_trait;
use log::{error, info};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

use crate::core::profile::{BaseActiveConfig, BasePassiveConfig};
use crate::net::proxy::{InputProxy, OutProxyStarter, OutputProxy, ProxyInfo, ProxyReader, ProxyWriter};
use crate::socks::socks5_connector::{Sock5ClientConnector, Socks5Server};

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
            let (tcp_stream, _addr) = self.tcp_listener.accept().await?;
            let starter = match out_proxy.gen_connector() {
                Ok(n) => n,
                Err(_) => continue
            };
            tokio::task::spawn(async move {
                if let Err(e) = new_proxy(tcp_stream, starter).await {
                    error!("Socks5 proxy error. {}", e)
                };
            });
        }
    }
}


async fn new_proxy(mut input_stream: TcpStream, mut starter: Box<dyn OutProxyStarter>) -> io::Result<()> {
    let mut connector = Socks5Server::new(&mut input_stream);
    let info = connector.accept_check().await?;

    let (mut out_reader, mut out_writer) = starter.new_connection(info).await?;

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

//----------------------Socks5Active--------------------

pub struct Socks5Active {
    socket_addr: SocketAddr
}

impl Socks5Active {
    pub fn new(active: &BaseActiveConfig) -> io::Result<Self> {
        let ip_addr = IpAddr::from_str(&active.remote_host)
            .or_else(|e| Err(Error::new(ErrorKind::InvalidInput, e)))?;
        let socket_addr = SocketAddr::new(ip_addr, active.remote_port);
        Ok(Self { socket_addr })
    }
}

impl OutputProxy for Socks5Active {
    fn gen_connector(&mut self) -> io::Result<Box<dyn OutProxyStarter>> {
        let starter = Socks5OutProxyStarter { socket_addr: self.socket_addr.clone() };
        Ok(Box::new(starter))
    }
}

struct Socks5OutProxyStarter {
    socket_addr: SocketAddr
}

#[async_trait]
impl OutProxyStarter for Socks5OutProxyStarter {
    async fn new_connection(&mut self, proxy_info: ProxyInfo) -> io::Result<(Box<dyn ProxyReader>, Box<dyn ProxyWriter>)> {
        let mut tcp_stream = TcpStream::connect(self.socket_addr.clone()).await?;
        let mut connector = Sock5ClientConnector::new(&mut tcp_stream);
        connector.try_connect(&proxy_info).await?;
        let (half_reader, half_writer) = tcp_stream.into_split();
        let reader = Socks5Redaer::new(half_reader);
        let writer = Socks5Writer::new(half_writer);
        Ok((Box::new(reader), Box::new(writer)))
    }
}

//--------------------------SOCKS5_READER_AND_WRITER-----------------------

struct Socks5Redaer {
    read_half: OwnedReadHalf,
    buffer: Box<[u8]>,
}

impl Socks5Redaer {
    pub fn new(read_half: OwnedReadHalf) -> Self {
        Self { read_half, buffer: vec![0u8; 32 * 1024].into_boxed_slice() }
    }
}

#[async_trait]
impl ProxyReader for Socks5Redaer {
    async fn read(&mut self) -> io::Result<&mut [u8]> {
        let read_size = self.read_half.read(&mut self.buffer).await?;
        if read_size == 0 { return Err(Error::new(ErrorKind::InvalidInput, "Socks5 read size is 0.")); }
        Ok(&mut self.buffer[..read_size])
    }

    async fn shutdown(&mut self) -> io::Result<()> {
        Ok(())
    }
}

struct Socks5Writer {
    write_half: OwnedWriteHalf,
}

impl Socks5Writer {
    pub fn new(write_half: OwnedWriteHalf) -> Self {
        Self { write_half }
    }
}

#[async_trait]
impl ProxyWriter for Socks5Writer {
    async fn write(&mut self, raw_data: &mut [u8]) -> io::Result<()> {
        self.write_half.write_all(raw_data).await
    }

    async fn shutdown(&mut self) -> io::Result<()> {
        self.write_half.shutdown().await
    }
}

