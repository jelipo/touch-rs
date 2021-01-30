use std::borrow::BorrowMut;
use std::io;

use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt, ErrorKind};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;

use crate::net::AddressType;
use crate::net::dns::DnsClient;
use crate::net::proxy::{OutProxyStarter, OutputProxy, ProxyInfo, ProxyReader, ProxyWriter};
use crate::util::address::Address;
use std::io::Error;
use std::net::SocketAddr;

pub struct RawActive {
    dns: Option<DnsClient>
}

/// Send raw data to dest server
impl RawActive {
    /// Init raw active.
    pub fn new(dns_config: Option<String>) -> io::Result<Self> {
        let dns = match dns_config {
            Some(dns_str) => Some(DnsClient::new(dns_str)?),
            None => None
        };
        Ok(Self { dns })
    }
}

impl OutputProxy for RawActive {
    fn gen_connector(&mut self) -> io::Result<Box<dyn OutProxyStarter>> {
        Ok(Box::new(RawOutProxyStarter { dns: self.dns.clone() }))
    }
}

pub struct RawOutProxyStarter {
    dns: Option<DnsClient>
}

#[async_trait]
impl OutProxyStarter for RawOutProxyStarter {
    async fn new_connection(&mut self, proxy_info: ProxyInfo) ->
    io::Result<(Box<dyn ProxyReader>, Box<dyn ProxyWriter>)> {
        let tcpstream = if (proxy_info.address_type == AddressType::Domain) && self.dns.is_some() {
            let client = self.dns.as_mut().unwrap();
            let ip_addr = client.query(&proxy_info.address).await
                .ok_or(Error::new(ErrorKind::InvalidInput, "Unknow host"))?;
            TcpStream::connect((ip_addr, proxy_info.port)).await?
        } else {
            Address::new_connect(&proxy_info.address, proxy_info.port, &proxy_info.address_type).await?
        };
        let (read_half, write_half) = tcpstream.into_split();
        let writer = RawProxyWriter::new(write_half);
        let reader = RawProxyReader::new(read_half);
        Ok((Box::new(reader), Box::new(writer)))
    }
}

pub struct RawProxyReader {
    read_half: OwnedReadHalf,
    buf: Box<[u8]>,
}

impl RawProxyReader {
    pub fn new(read_half: OwnedReadHalf) -> Self {
        Self {
            read_half,
            buf: vec![0u8; 32 * 1024].into_boxed_slice(),
        }
    }
}

#[async_trait]
impl ProxyReader for RawProxyReader {
    async fn read(&mut self) -> io::Result<&mut [u8]> {
        let size = self.read_half.read(self.buf.borrow_mut()).await?;
        Ok(&mut self.buf[..size])
    }

    async fn shutdown(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub struct RawProxyWriter {
    write_half: OwnedWriteHalf
}

impl RawProxyWriter {
    pub fn new(write_half: OwnedWriteHalf) -> Self {
        Self { write_half }
    }
}

#[async_trait]
impl ProxyWriter for RawProxyWriter {
    async fn write(&mut self, raw_data: &mut [u8]) -> io::Result<()> {
        self.write_half.write_all(raw_data).await
    }

    async fn shutdown(&mut self) -> io::Result<()> {
        self.write_half.shutdown().await
    }
}