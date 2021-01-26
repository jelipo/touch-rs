use std::net::{SocketAddr};
use std::io;
use crate::net::proxy::{OutputProxy, OutProxyStarter, ProxyInfo, ProxyReader, ProxyWriter};
use crate::util::address::Address;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::io::AsyncWriteExt;


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


impl OutputProxy for RawActive {
    fn gen_starter(&mut self) -> io::Result<Box<dyn OutProxyStarter>> {
        Ok(Box::new(RawOutProxyStarter {
            dns: self.dns.clone()
        }))
    }
}

pub struct RawOutProxyStarter {
    dns: Option<SocketAddr>
}

impl OutProxyStarter for RawOutProxyStarter {
    async fn new_connect(&mut self, proxy_info: ProxyInfo) ->
    io::Result<(Box<dyn ProxyReader>, Box<dyn ProxyWriter>)> {
        let tcpstream = Address::new_connect(
            &proxy_info.address, proxy_info.port, &proxy_info.address_type).await?;
        let (read_half, write_half) = tcpstream.into_split();
    }
}

pub struct RawProxyReader {
    read_half: OwnedReadHalf
}

impl ProxyReader for RawProxyReader {
    async fn read(&mut self) -> io::Result<&mut [u8]> {
        unimplemented!()
    }

    async fn shutdown(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub struct RawProxyWriter {
    write_half: OwnedWriteHalf
}

impl ProxyWriter for RawProxyWriter {
    async fn write(&mut self, raw_data: &mut [u8]) -> io::Result<()> {
        unimplemented!()
    }

    async fn write_adderss(&mut self, info: &ProxyInfo) -> io::Result<()> {
        unimplemented!()
    }

    async fn shutdown(&mut self) -> io::Result<()> {
        self.write_half.shutdown().await
    }
}