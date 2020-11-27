use std::str::FromStr;

use async_std::io;
use async_std::io::{Error, ErrorKind};
use async_std::io::ReadExt;
use async_std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use async_std::prelude::*;
use async_trait::async_trait;
use log::{error, info, trace, warn};

use crate::core::profile::BasePassiveConfig;
use crate::net::proxy::{Closer, InputProxy, OutProxyStarter, OutputProxy, ProxyInfo, ProxyReader, ProxyWriter};
use crate::socks::consts::Socks5Header;
use crate::socks::socks5_connector::Socks5Connector;

pub struct Socks5Passive {
    tcp_listerner: TcpListener,
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
            tcp_listerner: tcp_listener,
            password: passive.password.clone(),
            out_proxy,
        })
    }
}


#[async_trait]
impl InputProxy for Socks5Passive {
    async fn start(&mut self) -> io::Result<()> {
        println!("Sock5 start listen");
        loop {
            let out_proxy = &mut self.out_proxy;
            let mut tcpstream: TcpStream = self.tcp_listerner.incoming().next().await.ok_or(
                io::Error::new(ErrorKind::InvalidInput, "")
            )??;
            let mut connector = Socks5Connector::new(&mut tcpstream);
            let info = match connector.check().await {
                Ok(info) => info,
                Err(_) => continue
            };
            let mut starter = match out_proxy.gen_starter(info) {
                Ok(n) => n,
                Err(_) => continue
            };
            async_std::task::spawn(async move {
                if let Err(e) = new_proxy(&mut tcpstream, starter).await {
                    error!("Socks5 proxy error. {}", e)
                };
            });
        }
    }
}


async fn new_proxy(input_stream: &mut TcpStream, mut starter: Box<dyn OutProxyStarter + Send>) -> io::Result<()> {
    let (mut out_reader,
        mut out_writer,
        mut closer) = starter.new_connect().await?;
    let input_read = input_stream.clone();
    let input_write = input_stream.clone();
    let reader = async {
        write(input_write, out_reader).await
    };
    let writer = async {
        read(input_read, out_writer).await
    };
    // Wait for two future done.
    let size = reader.race(writer).await;
    input_stream.shutdown(Shutdown::Both);
    closer.shutdown()
}


async fn read(mut input_read: TcpStream, mut out_writer: Box<dyn ProxyWriter + Send>) -> usize {
    let mut buf = [0u8; 1024];
    let mut total = 0;
    loop {
        let size = match input_read.read(&mut buf).await {
            Ok(n) => n,
            Err(_) => break,
        };
        if size == 0 { break; }
        total = total + size;
        if out_writer.write(&buf[0..size]).await.is_err() { break; }
    }
    total
}

async fn write(mut input_write: TcpStream, mut out_reader: Box<dyn ProxyReader + Send>) -> usize {
    let mut total = 0;
    loop {
        let data = match out_reader.read().await {
            Ok(n) => n,
            Err(_) => break,
        };
        if data.len() == 0 { break; }
        total = total + data.len();
        if input_write.write_all(data.as_ref()).await.is_err() { break; };
    }
    total
}