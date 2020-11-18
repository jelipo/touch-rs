use async_std::net::{SocketAddr, ToSocketAddrs};
use async_trait::async_trait;

use crate::net::AddressType;

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

#[async_trait(? Send)]
pub trait InputProxy {
    /// Start proxy.
    async fn start(&mut self);
}

pub struct ProxyInfo {
    pub address_type: AddressType,
    pub address: Box<Vec<u8>>,
    pub port: u16,
}

