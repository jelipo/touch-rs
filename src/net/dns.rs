use std::io;
use std::io::Error;
use std::io::ErrorKind;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

use anyhow::{anyhow, Result};
use tokio::net::UdpSocket;
use trust_dns_client::client::{AsyncClient, ClientHandle};
use trust_dns_client::rr::{DNSClass, Name, RecordType};
use trust_dns_client::udp::UdpClientStream;

#[derive(Clone)]
pub struct DnsClient {
    client: AsyncClient
}

impl DnsClient {
    pub async fn new(dns_addr: String) -> io::Result<Self> {
        let mut dns_addr = dns_addr.clone();
        if !dns_addr.contains(":") {
            dns_addr = format!("{}:{}", dns_addr, "53")
        }
        let addr = SocketAddr::from_str(dns_addr.as_str())
            .or_else(|e| Err(Error::new(ErrorKind::InvalidInput, e)))?;
        let stream = UdpClientStream::<UdpSocket>::new(addr);
        let (client, _) = AsyncClient::connect(stream).await?;
        Ok(Self { client })
    }

    pub async fn query(&mut self, domain: &str) -> Result<IpAddr> {
        let response = self.client.query(
            Name::from_str(domain)?, DNSClass::IN, RecordType::A).await?;
        let answers = response.answers();
        if answers.len() == 0 {
            Err(anyhow!("Unknow host:{}",domain))
        } else {
            Ok(answers[0].rdata().to_ip_addr().unwrap())
        }
    }
}