use std::net::{Ipv6Addr, SocketAddrV6};
use crate::net::AddressType;
use std::borrow::Borrow;

pub struct Socks5 {}

impl Socks5 {
    /// # Return value
    /// - `String` Host address
    /// - `u8` Number of bytes read
    pub fn read_to_socket_addrs(bytes: &[u8]) -> (String, usize) {
        let addr_type = bytes[0];
        match addr_type {
            0x01 => {
                let port = u16::from_be_bytes([bytes[5], bytes[6]]);
                (format!("{}.{}.{}.{}:{}", bytes[1], bytes[2], bytes[3], bytes[4], port), 7)
            }
            0x04 => {
                let mut ipv6_arr = [0u8; 16];
                ipv6_arr.copy_from_slice(&bytes[1..17]);
                let port = u16::from_be_bytes([bytes[17], bytes[18]]);
                let v6 = SocketAddrV6::new(Ipv6Addr::from(ipv6_arr), port, 0, 0);
                (v6.to_string(), 19)
            }
            _ => {
                let domain_len = bytes[1] as usize;
                let cow = String::from_utf8_lossy(&bytes[2..(domain_len + 2)]);
                let port = u16::from_be_bytes([bytes[domain_len + 2], bytes[domain_len + 3]]);
                (format!("{}:{}", cow, port), 4 + domain_len)
            }
        }
    }

    pub fn socks5_addr_arr(host: &Vec<u8>, port: u16, addr_type: &AddressType) -> Box<[u8]> {
        match addr_type {
            AddressType::IPv4 => {
                let port_byte: [u8; 2] = port.to_be_bytes();
                [1, host[0], host[1], host[2], host[3], port_byte[0], port_byte[1]].into()
            }
            AddressType::Domain => {
                let host_len = host.len();
                let mut vec = vec![0u8; host_len + 4];
                vec[0] = 0x03;
                vec[1] = host_len as u8;
                vec[2..host_len + 2].copy_from_slice(host.borrow());
                vec[host_len + 2..host_len + 4].copy_from_slice(&port.to_be_bytes());
                println!("{:?}", vec);
                vec.into()
            }
            AddressType::IPv6 => {
                let mut arr = [4u8; 19];
                arr[1..17].copy_from_slice(host.borrow());
                arr[17..19].copy_from_slice(&port.to_be_bytes());
                arr.into()
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::socks::socks5::Socks5;

    #[test]
    fn test() {
        let mut bytes = [0u8; 128];
        bytes[0] = 1;
        let string = Socks5::read_to_socket_addrs(&bytes);
        println!("{}", string.0);
    }
}