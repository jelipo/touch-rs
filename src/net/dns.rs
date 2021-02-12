use std::io;
use std::io::Error;
use std::io::ErrorKind;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

use anyhow::{anyhow, Result};
use tokio::net::UdpSocket;
use trust_dns_resolver::config::{NameServerConfig, ResolverConfig, ResolverOpts};
use trust_dns_resolver::{TokioAsyncResolver, Name};
use trust_dns_resolver::proto::serialize::binary::BinDecodable;

#[derive(Clone)]
pub struct DnsClient {
    resolver: TokioAsyncResolver
}

impl DnsClient {
    /// Creat a new DNS client.
    /// Use UDP to query ip
    pub fn new(dns_addr: String) -> io::Result<Self> {
        let mut dns_addr = dns_addr.clone();
        if !dns_addr.contains(":") {
            dns_addr = format!("{}:{}", dns_addr, "53")
        }
        let addr = SocketAddr::from_str(dns_addr.as_str())
            .or_else(|e| Err(Error::new(ErrorKind::InvalidInput, e)))?;
        let mut config = ResolverConfig::new();
        config.add_name_server(NameServerConfig {
            socket_addr: addr,
            protocol: Default::default(),
            tls_dns_name: None,
            trust_nx_responses: false,
        });
        let resolver = TokioAsyncResolver::tokio(config, ResolverOpts::default())?;
        Ok(Self { resolver })
    }

    /// Query IP of the domain name.
    pub async fn query(&self, domain: &[u8]) -> Option<IpAddr> {
        let name = Name::from_bytes(domain).ok()?;
        let response = self.resolver.lookup_ip(name).await.ok()?;
        response.iter().next()
    }
}