use async_trait::async_trait;

use crate::net::AddressType;
use std::net::ToSocketAddrs;
use std::io;

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

#[async_trait]
pub trait InputProxy {
    /// Start proxy.
    async fn start(&mut self) -> io::Result<()>;
}


pub trait OutputProxy: Send {
    /// Creat a new output proxy starter.
    fn gen_starter(&mut self) -> io::Result<Box<dyn OutProxyStarter>>;
}

#[async_trait]
pub trait OutProxyStarter: Send {
    async fn new_connect(&mut self, proxy_info: ProxyInfo) ->
    io::Result<(Box<dyn ProxyReader>, Box<dyn ProxyWriter>)>;
}

#[async_trait]
pub trait Closer: Send {
    async fn shutdown(&mut self) -> io::Result<()>;
}

#[async_trait]
pub trait ProxyReader: Send {
    async fn read(&mut self) -> io::Result<&mut [u8]>;

    async fn shutdown(&mut self) -> io::Result<()>;
}

#[async_trait]
pub trait ProxyWriter: Send {
    async fn write(&mut self, raw_data: &mut [u8]) -> io::Result<()>;

    async fn write_adderss(&mut self, info: &ProxyInfo) -> io::Result<()>;

    async fn shutdown(&mut self) -> io::Result<()>;
}


#[derive(Clone, Debug)]
pub struct ProxyInfo {
    pub address_type: AddressType,
    pub address: Box<Vec<u8>>,
    pub port: u16,
}