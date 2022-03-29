use std::io;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6};

use tokio::net::TcpStream;

use crate::net::AddressType;

pub struct Address {}

impl Address {
    pub fn ip_str(ip_data: &[u8], port: u16, addr_type: &AddressType) -> String {
        match addr_type {
            AddressType::IPv4 => {
                let ipv4_addr = Ipv4Addr::new(ip_data[0], ip_data[1], ip_data[2], ip_data[3]);
                SocketAddrV4::new(ipv4_addr, port).to_string()
            }
            AddressType::Domain => format!("{}:{}", String::from_utf8_lossy(ip_data), port),
            AddressType::IPv6 => {
                let mut ip_arr = [0u8; 16];
                ip_arr.copy_from_slice(&ip_data[0..16]);
                let ipv6_addr = Ipv6Addr::from(ip_arr);
                SocketAddrV6::new(ipv6_addr, port, 0, 0).to_string()
            }
        }
    }

    /// Creat a new TCP connect with [AddressType]
    pub async fn new_connect(ip_data: &[u8], port: u16, addr_type: &AddressType) -> io::Result<TcpStream> {
        match addr_type {
            AddressType::IPv4 => {
                let ipv4_addr = Ipv4Addr::new(ip_data[0], ip_data[1], ip_data[2], ip_data[3]);
                TcpStream::connect(SocketAddrV4::new(ipv4_addr, port)).await
            }
            AddressType::Domain => {
                let addr = format!("{}:{}", String::from_utf8_lossy(ip_data), port);
                TcpStream::connect(addr.as_str()).await
            }
            AddressType::IPv6 => {
                let mut ip_arr = [0u8; 16];
                ip_arr.copy_from_slice(&ip_data[..16]);
                let ipv6_addr = Ipv6Addr::from(ip_arr);
                TcpStream::connect(SocketAddrV6::new(ipv6_addr, port, 0, 0)).await
            }
        }
    }
}
