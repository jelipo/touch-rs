use std::str::FromStr;

use async_std::io;
use async_std::io::{Error, ErrorKind};
use async_std::io::ReadExt;
use async_std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6, TcpListener, TcpStream, ToSocketAddrs};
use async_std::stream::StreamExt;
use async_trait::async_trait;
use log::{error, info, trace, warn};
use serde::export::Result::Ok;

use crate::core::profile::BasePassiveConfig;
use crate::net::AddressType;
use crate::net::proxy::{InputProxy, ProxyInfo};
use crate::socks::consts::Socks5Header;
use crate::socks::socks5_connector::Socks5Connector;
use std::borrow::{Borrow, BorrowMut};

pub struct Socks5Passive {
    tcp_listerner: TcpListener,
    password: Option<String>,
}

impl Socks5Passive {
    /// Init Socks5 Passive. And try to bind host and port
    pub async fn new(passive: &BasePassiveConfig) -> io::Result<Self> {
        let adde_str = format!("{}:{}", &passive.local_host, passive.local_port);
        let addr = SocketAddr::from_str(adde_str.as_str()).or(
            Err(Error::new(ErrorKind::InvalidInput, "Error address"))
        );
        let tcp_listener = TcpListener::bind(addr?).await?;
        info!("Socks5 bind in {}", adde_str);
        Ok(Self {
            tcp_listerner: tcp_listener,
            password: passive.password.clone(),
        })
    }

    async fn new_proxy(&mut self, input_stream: &mut TcpStream, info: ProxyInfo) -> io::Result<()> {
        //let handle = async_std::task::spawn(move || {});
        let stream = input_stream.clone();
        Ok(())
    }
}

#[async_trait(? Send)]
impl InputProxy for Socks5Passive {
    async fn start(&mut self) {
        loop {
            let socks5_passive = self.borrow_mut();
            let mut incoming = socks5_passive.tcp_listerner.incoming();
            if let Some(Ok(mut tcpstream)) = incoming.next().await {
                let mut connector = Socks5Connector::new(&mut tcpstream);
                match connector.check().await {
                    Ok(proxy_info) => {
                        if let Err(e) = self.new_proxy(&mut tcpstream, proxy_info).await {
                            error!("Socks5 proxy error. {}", e)
                        }
                    }
                    Err(e) => error!("Sock5 checked failed.{}", e)
                }
            } else {
                trace!("Connect error.")
            }
        }
    }
}


