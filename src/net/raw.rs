use std::borrow::Borrow;
use std::io;
use std::io::Error;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt, ErrorKind};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;

use crate::net::dns::DnsClient;
use crate::net::proxy::{OutProxyStarter, OutputProxy, ProxyInfo, ProxyReader, ProxyWriter};
use crate::net::AddressType;
use crate::util::address::Address;

pub struct RawActive {
    dns: Arc<Option<DnsClient>>,
}

/// Send raw data to dest server
impl RawActive {
    /// Init raw active.
    pub fn new(dns_config: Option<String>) -> io::Result<Self> {
        let dns = dns_config.and_then(|dns_str| DnsClient::new(dns_str).ok());
        Ok(Self { dns: Arc::new(dns) })
    }
}

impl OutputProxy for RawActive {
    fn gen_connector(&mut self) -> io::Result<Box<dyn OutProxyStarter>> {
        Ok(Box::new(RawOutProxyStarter { dns: self.dns.clone() }))
    }
}

pub struct RawOutProxyStarter {
    dns: Arc<Option<DnsClient>>,
}

#[async_trait]
impl OutProxyStarter for RawOutProxyStarter {
    async fn new_connection(&mut self, proxy_info: ProxyInfo) -> io::Result<(Box<dyn ProxyReader>, Box<dyn ProxyWriter>)> {
        let tcp_stream = if proxy_info.address_type == AddressType::Domain {
            let domain_str = String::from_utf8_lossy(&proxy_info.address);
            if let Some(client) = self.dns.borrow() {
                // Query domain IP address
                let ip_addr = client.query(&proxy_info.address).await
                    .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "unknown host"))?;
                TcpStream::connect((ip_addr, proxy_info.port)).await?
            } else {
                let address = format!("{}:{}", domain_str, proxy_info.port);
                TcpStream::connect(&address).await?
            }
        } else {
            Address::new_connect(&proxy_info.address, proxy_info.port, &proxy_info.address_type).await?
        };
        let (read_half, write_half) = tcp_stream.into_split();
        let writer = RawProxyWriter::new(write_half);
        let reader = RawProxyReader::new(read_half);
        Ok((Box::new(reader), Box::new(writer)))
    }
}

pub struct RawProxyReader {
    read_half: OwnedReadHalf,
    buf: Vec<u8>,
}

impl RawProxyReader {
    pub fn new(read_half: OwnedReadHalf) -> Self {
        Self {
            read_half,
            buf: vec![0u8; 32 * 1024],
        }
    }
}

#[async_trait]
impl ProxyReader for RawProxyReader {
    async fn read(&mut self) -> io::Result<&mut [u8]> {
        let size = self.read_half.read(&mut self.buf).await?;
        Ok(&mut self.buf[..size])
    }

    async fn shutdown(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub struct RawProxyWriter {
    write_half: OwnedWriteHalf,
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
