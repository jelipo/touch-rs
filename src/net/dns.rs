use std::io;
use std::io::Error;
use std::io::ErrorKind;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

use anyhow::{anyhow, Result};
use tokio::net::UdpSocket;
use trust_dns_client::client::{AsyncClient, ClientHandle};
use trust_dns_client::rr::{DNSClass, Name, RecordType};
use trust_dns_client::serialize::binary::BinDecodable;
use trust_dns_client::udp::UdpClientStream;

#[derive(Clone)]
pub struct DnsClient {
    socket_addr: SocketAddr,
    client: Option<AsyncClient>,
}

impl DnsClient {
    /// Creat a new DNS client.
    /// Uses UDP to query
    pub fn new(dns_addr: String) -> io::Result<Self> {
        let mut dns_addr = dns_addr.clone();
        if !dns_addr.contains(":") {
            dns_addr = format!("{}:{}", dns_addr, "53")
        }
        let addr = SocketAddr::from_str(dns_addr.as_str())
            .or_else(|e| Err(Error::new(ErrorKind::InvalidInput, e)))?;
        Ok(Self { socket_addr: addr, client: None })
    }

    /// Query IP of the doamin name.
    pub async fn query(&mut self, domain: &[u8]) -> Option<IpAddr> {
        if self.client.is_none() {
            let stream = UdpClientStream::<UdpSocket>::new(self.socket_addr);
            let (client, _) = AsyncClient::connect(stream).await.ok()?;
            self.client = Some(client);
        }
        let client = self.client.as_mut().unwrap();
        let response = client.query(
            Name::from_bytes(domain).ok()?, DNSClass::IN, RecordType::A).await.ok()?;
        let answers = response.answers();
        if answers.len() == 0 { None } else { Some(answers[0].rdata().to_ip_addr().unwrap()) }
    }
}