use std::io;

use async_trait::async_trait;

use crate::net::AddressType;

#[async_trait]
pub trait InputProxy {
    /// Start proxy.
    async fn start(&mut self) -> io::Result<()>;
}

pub trait OutputProxy: Send {
    /// Creat a new output proxy connector.
    fn gen_connector(&mut self) -> io::Result<Box<dyn OutProxyStarter>>;
}

#[async_trait]
pub trait OutProxyStarter: Send {
    /// Creat a new OUT_PROXY connection.
    async fn new_connection(&mut self, proxy_info: ProxyInfo) -> io::Result<(Box<dyn ProxyReader>, Box<dyn ProxyWriter>)>;
}

#[async_trait]
pub trait ProxyReader: Send {
    async fn read(&mut self) -> io::Result<&mut [u8]>;

    async fn shutdown(&mut self) -> io::Result<()>;
}

#[async_trait]
pub trait ProxyWriter: Send {
    async fn write(&mut self, raw_data: &mut [u8]) -> io::Result<()>;

    async fn shutdown(&mut self) -> io::Result<()>;
}

#[derive(Clone, Debug)]
pub struct ProxyInfo {
    pub address_type: AddressType,
    pub address: Vec<u8>,
    pub port: u16,
}
