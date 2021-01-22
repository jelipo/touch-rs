use std::str::FromStr;


use async_trait::async_trait;
use log::{error, info};

use crate::core::profile::BasePassiveConfig;
use crate::net::proxy::{Closer, InputProxy, OutProxyStarter, OutputProxy, ProxyReader, ProxyWriter, ProxyInfo};
use crate::socks::socks5_connector::Socks5Connector;
use std::net::{SocketAddr, Ipv4Addr};
use std::io;
use tokio::net::TcpStream;
use crate::net::AddressType;
use crate::util::address::Address;

pub struct RawActive {
    dns: Option<SocketAddr>
}

/// Send raw data to dest server
impl RawActive {
    /// Init raw active.
    pub async fn new(dns: Option<SocketAddr>) -> io::Result<Self> {
        Ok(Self { dns })
    }
}


// impl OutputProxy for RawActive {
//     fn gen_starter(&mut self) -> io::Result<Box<dyn OutProxyStarter>> {
//         Ok(Box::new(RawOutProxyStarter {
//             dns: self.dns.clone()
//         }))
//     }
// }
//
// pub struct RawOutProxyStarter {
//     dns: Option<SocketAddr>
// }
