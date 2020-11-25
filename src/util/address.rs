use crate::net::AddressType;
use std::net::{Ipv4Addr, SocketAddrV4};
use async_std::net::{Ipv6Addr, SocketAddrV6};

pub struct Address {}

impl Address {
    pub fn ip_str(ip_data: &[u8], port: u16, addr_type: &AddressType) -> String {
        match addr_type {
            AddressType::IPv4 => {
                let ipv4_addr = Ipv4Addr::new(ip_data[0], ip_data[1], ip_data[2], ip_data[3]);
                SocketAddrV4::new(ipv4_addr, port).to_string()
            }
            AddressType::Domain => String::from_utf8_lossy(ip_data).to_string(),
            AddressType::IPv6 => {
                let mut ip_arr = [0u8; 16];
                ip_arr.copy_from_slice(&ip_data[0..16]);
                let ipv6_addr = Ipv6Addr::from(ip_arr);
                SocketAddrV6::new(ipv6_addr, port, 0, 0).to_string()
            }
        }
    }
}