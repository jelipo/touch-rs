use std::borrow::Borrow;
use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;

use async_std::io;
use async_std::io::{Error, ErrorKind};
use async_std::io::ReadExt;
use async_std::net::{SocketAddr, SocketAddrV4, TcpListener, TcpStream};
use async_std::prelude::*;
use async_std::stream::StreamExt;
use async_std::task::JoinHandle;
use async_trait::async_trait;
use log::{error, info, trace, warn};
use serde::export::Result::Ok;

use crate::core::profile::BasePassiveConfig;
use crate::net::proxy::{InputProxy, OutputProxy, ProxyInfo, ProxyReader, ProxyWriter};
use crate::socks::consts::Socks5Header;
use crate::socks::socks5_connector::Socks5Connector;

pub struct Socks5Passive {
    tcp_listerner: TcpListener,
    password: Option<String>,
    out_proxy: Box<dyn OutputProxy>,
}

impl Socks5Passive {
    /// Init Socks5 Passive. And try to bind host and port
    pub async fn new(passive: &BasePassiveConfig, out_proxy: Box<dyn OutputProxy>) -> io::Result<Self> {
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


#[async_trait(? Send)]
impl InputProxy for Socks5Passive {
    async fn start(&mut self) -> io::Result<()> {
        let rc = Rc::new(RefCell::new(self));
        loop {
            let tcp: TcpStream = self.tcp_listerner.incoming().next().await.ok_or(
                Err(Error::new(ErrorKind::InvalidInput, ""))
            )?;
            if let Some(Ok(mut tcpstream)) = rc.borrow_mut().tcp_listerner.incoming().next().await {
                let mut connector = Socks5Connector::new(&mut tcpstream);
                match connector.check().await {
                    Ok(proxy_info) => {
                        let rc1 = Rc::clone(&rc);
                        if let Err(e) = new_proxy(&mut rc1.borrow_mut().out_proxy, &mut tcpstream, proxy_info).await {
                            error!("Socks5 proxy error. {}", e)
                        };
                    }
                    Err(e) => error!("Sock5 checked failed.{}", e)
                };
            } else {
                trace!("Connect error.")
            };
        }
    }
}


async fn new_proxy(out_proxy: &mut Box<dyn OutputProxy>, input_stream: &mut TcpStream, info: ProxyInfo) -> io::Result<()> {
    let (mut out_reader, mut out_writer) =
        out_proxy.new_connect(info).await;
    let mut input_read = input_stream.clone();
    let mut input_write = input_stream.clone();
    let handle1 = async_std::task::spawn(async move {
        read(input_read, out_writer).await
    });
    let handle2 = async_std::task::spawn(async move {
        write(input_write, out_reader).await
    });
    let x = handle1.await;
    let x = handle2.await;
    Ok(())
}


async fn read(mut input_read: TcpStream, mut out_writer: Box<dyn ProxyWriter + Send>) -> io::Result<u64> {
    let mut buf = [0u8; 4096];
    let mut total = 0u64;
    loop {
        let size = input_read.read(&mut buf).await?;
        if size == 0 { break; }
        total = total + size;
        out_writer.write(&buf[0..size]).await;
    }
    Ok(total)
}

async fn write(mut input_write: TcpStream, mut out_reader: Box<dyn ProxyReader + Send>) -> io::Result<u64> {
    let mut total = 0u64;
    loop {
        let data = out_reader.read().await?;
        total = total + data.len();
        input_write.write_all(data.as_slice()).await?;
    }
    Ok(total)
}