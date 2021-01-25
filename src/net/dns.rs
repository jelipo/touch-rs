use std::io;
use std::io::Error;
use std::io::ErrorKind;

use tokio::runtime::Runtime;
use trust_dns_client::client::{AsyncClient, ClientHandle};
use trust_dns_client::proto::udp::{UdpClientConnect};
use trust_dns_client::udp::{UdpClientConnection, UdpClientStream};
use trust_dns_client::rr::{Name, DNSClass, RecordType};
use std::str::FromStr;
use trust_dns_client::op::DnsResponse;
use tokio::net::UdpSocket;
use std::net::SocketAddr;

pub struct DnsClient {
    client: AsyncClient
}

impl DnsClient {
    pub async fn new(dns_addr: String) -> io::Result<Self> {
        let mut dns_addr = dns_addr.clone();
        if !dns_addr.contains(":") {
            dns_addr = format!("{}:{}", dns_addr, "53")
        }
        let addr = match SocketAddr::from_str(dns_addr.as_str()) {
            Ok(addr) => addr,
            Err(e) => return Err(Error::new(ErrorKind::InvalidInput, e))
        };
        let stream = UdpClientStream::<UdpSocket>::new(addr);
        let (client, _) = AsyncClient::connect(stream).await?;
        Ok(Self { client })
    }

    pub async fn query(&mut self, domain: String) -> io::Result<DnsResponse> {
        let response = self.client.query(
            Name::from_str(domain.as_str()).unwrap(), DNSClass::IN, RecordType::A).await.unwrap();
        Ok(response)
    }
}
