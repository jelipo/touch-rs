use async_std::net::{SocketAddr, ToSocketAddrs};
use async_trait::async_trait;

use crate::net::AddressType;
use async_std::io;

pub struct Proxy<A: ToSocketAddrs> {
    addredd_type: AddressType,
    socket_addr: Box<A>,
}

impl<A: ToSocketAddrs> Proxy<A> {
    fn new(addredd_type: AddressType, socket_addr: A) -> Self {
        Proxy {
            addredd_type,
            socket_addr: Box::new(socket_addr),
        }
    }
}

#[async_trait(? Send)]
pub trait InputProxy {
    /// Start proxy.
    async fn start(&mut self);
}

#[async_trait(? Send)]
pub trait OutputProxy {
    /// Creat a new connect.
    async fn new_connect(&mut self, proxy_info: ProxyInfo) -> Box<dyn ProxyStream>;
}

#[async_trait(? Send)]
pub trait ProxyStream {
    async fn read(&mut self) -> io::Result<Vec<u8>>;

    async fn write(&mut self, raw_data: &[u8]) -> io::Result<()>;
}

pub struct ProxyInfo {
    pub address_type: AddressType,
    pub address: Box<Vec<u8>>,
    pub port: u16,
}