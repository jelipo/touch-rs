use async_std::io;
use async_std::net::ToSocketAddrs;
use async_trait::async_trait;

use crate::net::AddressType;

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
    async fn start(&mut self) -> io::Result<()>;
}

#[async_trait]
pub trait OutputProxy {
    /// Creat a new connect.
    async fn new_connect(&mut self, proxy_info: ProxyInfo) ->
    io::Result<(Box<dyn ProxyReader + Send>, Box<dyn ProxyWriter + Send>, Box<dyn Closer + Send>)>;
}

#[async_trait]
pub trait Closer {
    fn shutdown(&mut self) -> io::Result<()>;
}

#[async_trait]
pub trait ProxyReader {
    async fn read(&mut self) -> io::Result<Vec<u8>>;
}

#[async_trait]
pub trait ProxyWriter {
    async fn write(&mut self, raw_data: &[u8]) -> io::Result<()>;
}


pub struct ProxyInfo {
    pub address_type: AddressType,
    pub address: Box<Vec<u8>>,
    pub port: u16,
}