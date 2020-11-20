use std::cell::RefCell;
use std::io::{Read, Write};
use std::rc::Rc;
use std::str::FromStr;
use async_std::prelude::*;
use async_std::io;
use async_std::io::{Error, ErrorKind};
use async_std::io::ReadExt;
use async_std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6, TcpListener, ToSocketAddrs, TcpStream};
use async_std::stream::StreamExt;
use async_std::sync::Arc;
use async_std::task::JoinHandle;
use async_trait::async_trait;
use log::{error, info, trace, warn};
use serde::export::Result::Ok;

use crate::core::profile::BasePassiveConfig;
use crate::net::AddressType;
use crate::net::proxy::{InputProxy, OutputProxy, ProxyInfo};
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

    async fn new_proxy(&mut self, input_stream: &mut TcpStream, info: ProxyInfo) -> io::Result<()> {

        let mut out_proxy_stream = self.out_proxy.new_connect(info).await;
        let out_stream_arc = Arc::new(out_proxy_stream.as_mut());
        let mut input_read = input_stream.clone();
        let mut input_write = input_stream.clone();
        let mut output_read = out_stream_arc.clone();
        let mut output_write = out_stream_arc.clone();

        let mut handle1: JoinHandle<()> = async_std::task::spawn(async move {
            let mut buf = [0u8; 1024];
            loop {
                let size = input_read.read(&mut buf).await.unwrap();
                output_write.write(&buf[0..size]).await.unwrap();
            }
        });
        let mut handle2: JoinHandle<()> = async_std::task::spawn(async move {
            loop {
                let data = output_read.read().await.unwrap();
                input_write.write_all(data.as_slice()).await.unwrap();
            }
        });
        let x = handle1.await;
        let x = handle2.await;
        Ok(())
    }
}

#[async_trait(? Send)]
impl InputProxy for Socks5Passive {
    async fn start(&mut self) {
        let rc = Rc::new(RefCell::new(self));
        loop {
            if let Some(Ok(mut tcpstream)) = rc.borrow().tcp_listerner.incoming().next().await {
                let mut connector = Socks5Connector::new(&mut tcpstream);
                match connector.check().await {
                    Ok(proxy_info) => {
                        let rc1 = Rc::clone(&rc);
                        if let Err(e) = rc1.borrow_mut().new_proxy(&mut tcpstream, proxy_info).await {
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

