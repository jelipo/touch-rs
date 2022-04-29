use std::io;
use std::io::Error;
use std::io::ErrorKind;
use std::net::{IpAddr, SocketAddr};
use std::ops::Deref;
use std::str::FromStr;

use trust_dns_resolver::config::{NameServerConfig, ResolverConfig, ResolverOpts};
use trust_dns_resolver::TokioAsyncResolver;

#[derive(Clone)]
pub struct DnsClient {
    resolver: TokioAsyncResolver,
}

impl DnsClient {
    /// Creat a new DNS client.
    /// Use UDP to query ip
    pub fn new(dns_addr: String) -> io::Result<Self> {
        let mut dns_addr = dns_addr;
        if !dns_addr.contains(':') {
            dns_addr = format!("{}:{}", dns_addr, "53")
        }
        let addr = SocketAddr::from_str(dns_addr.as_str()).map_err(|e| Error::new(ErrorKind::InvalidInput, e))?;
        let mut config = ResolverConfig::new();
        config.add_name_server(NameServerConfig {
            socket_addr: addr,
            protocol: Default::default(),
            tls_dns_name: None,
            trust_nx_responses: false,
            bind_addr: None,
        });
        let resolver = TokioAsyncResolver::tokio(config, ResolverOpts::default())?;
        Ok(Self { resolver })
    }

    /// Query IP of the domain name.
    pub async fn query(&self, domain: &[u8]) -> Option<IpAddr> {
        let addr_str = String::from_utf8_lossy(domain);
        let addr_def = addr_str.deref();
        match IpAddr::from_str(addr_def) {
            Ok(addr) => Some(addr),
            Err(_) => {
                let response = self.resolver.lookup_ip(addr_def).await.ok()?;
                response.iter().next()
            }
        }
    }
}
